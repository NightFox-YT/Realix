; © Realix > FAT12: File Load
; (13.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/disk/read, kernel16/io/print_ctrl.asm

; ❗️ Требуется инициализация FAT12 через `fat12_init`
%include "filesystem/fat12/init.asm"

; Основные константы
%include 'shared/config.asm'

; Настройка DiskAPI: Read (Если не настроен)
%ifndef DISK_READ
%define DISK_READ bios_disk_read
%endif

; Предел длины цепочки кластеров (Aрхитектурный потолок FAT12)
FAT12_MAX_CHAIN equ 4084

; > Загрузка файла с диска в память
; Параметры:
;  - si: смещение адреса имени файла (11 символов, формат 8.3)
;  - cx: сегмент назначения файла
;  - bx: смещение назначения файла
;  - di: адрес extra-таблицы защиты регионов памяти (0 - только core)
; Вывод:
; ❗️ Входной si (имя файла) при возврате не сохраняется
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка)
;  - si: смещение адреса сообщения об ошибке (0 - успех)
file_load:
    push ax
    push bx
    push cx
    push dx
    push di
    push es

    ; Настраиваем флаг направления и сегмент es
    cld
    xor ax, ax
    mov es, ax

    ; Сохраняем входные данные
    mov [filename_offset], si
    mov [dest_segment], cx
    mov [dest_offset], bx
    mov [extra_table_offset], di

.read_root_dir:
    ; Читаем корневой каталог в память
    mov ax, [root_dir_lba]
    mov cx, [root_dir_size]
    mov bx, FAT_BUFFER_ADDR
    call DISK_READ

    ; Подготовка к поиску файла в корневом каталоге
    xor bx, bx               ; Счётчик пройденных записей
    mov di, FAT_BUFFER_ADDR  ; Смещение текущей записи

.search:
    ; Подготовка к сравнению названий (до 11 символов)
    mov si, [filename_offset]
    mov cx, 11

    ; Сравниваем по символу названия файлов, сохраняя смещение записи
    ; > ds:si & es:di; si++, di++ до cx == 0
    push di
    repe cmpsb
    pop di
    je .found

    ; Переход к следующей записи
    add di, 32             ; Увеличиваем смещение на размер записи (32 байта)
    inc bx                 ; Увеличиваем индекс записи
    cmp bx, [dir_entries] 
    jb .search             ; Если не вышли за предел, продолжаем поиск

    ; Ошибка 1: Вышли за предел, => указанного файла нет
    mov si, err_not_found
    jmp .fail

.found:
    ; Обновление номера кластера (di - смещение записи корневого каталога)
    mov ax, [es:di + 26]    ; Поле первого кластера (Смещение 26 байтов)
    mov [file_cluster], ax  ; Обновляем переменную

    ; Линейный диапазон назначения: [start, end)
    ; > start = dest_segment * 16 + dest_offset, end = start + размер файла
    movzx eax, word [dest_segment]
    shl eax, 4
    movzx ebx, word [dest_offset]
    add ebx, eax

    ; Считаем end (Сохраняем размер файла для команд отображения)
    mov ecx, [es:di + 28]
    mov [file_size], ecx
    add ecx, ebx

    ; Проверка на пустой файл: Цепочки кластеров нет
    ; (0 - пусто, 1 - резерв)
    cmp word [file_cluster], 2
    jb .done

    ; Проверка core-таблицы (Запретна для любой загрузки)
    mov di, guard_core
    call check_table
    jc .guard_fail

    ; Проверка extra-таблицы, если передана (di != 0)
    mov di, [extra_table_offset]
    or di, di
    jz .read_fat
    call check_table
    jnc .read_fat

.guard_fail:
    ; Ошибка 3: Назначение файла пересекает защищённый регион
    mov si, err_guard
    jmp .fail

.read_fat:
    ; Сброс счётчика пройденных кластеров (Защита от петли FAT)
    mov word [chain_length], 0

    ; Читаем FAT в память
    mov ax, [reserved_sectors]
    mov cx, [sectors_per_fat]
    mov bx, FAT_BUFFER_ADDR
    call DISK_READ

    ; Установка сегмента и смещения для чтения файла
    mov bx, [dest_segment]
    mov es, bx
    mov bx, [dest_offset]

; Обработка FAT цепочки
.load_loop:
    ; Вычисление LBA кластера
    ; > LBA = (file_cluster - 2) * sectors_per_cluster + data_lba
    movzx cx, byte [sectors_per_cluster]
    mov ax, [file_cluster]
    sub ax, 2
    mul cx
    add ax, [data_lba]

    ; Чтение следующего кластера (cl содержит кол-во секторов)
    call DISK_READ

    ; Индикатор прогресса чтения ("кубики")
    call print_square_char

    ; Увеличиваем адрес смещения назначения на кол-во прочитанных байт
    movzx ax, byte [sectors_per_cluster]
    mul word [bytes_per_sector]
    add bx, ax
    jnc .load_loop_continue

    ; Сдвигаем es на 0x1000 параграфов (+64 КБ)
    mov ax, es
    add ax, 0x1000
    mov es, ax
    xor bx, bx

; Продолжение чтения файла
.load_loop_continue:
    ; Вычисляем байтовое смещение записи след. кластера
    ; > (cluster * 3 / 2), ax - смещение в байтах, dx - Cluster % 2
    mov ax, [file_cluster]
    mov cx, 3
    mul cx
    mov cx, 2
    div cx

    ; Считывание записи из таблицы FAT по индексу (ax)
    push es
    xor cx, cx
    mov es, cx

    mov si, FAT_BUFFER_ADDR
    add si, ax
    mov ax, [es:si]

    pop es

    ; Проверка чётности кластера
    or dx, dx
    jz .even_cluster

; Нечётный кластер (Оставляем старшие 12 бит)
.odd_cluster:
    shr ax, 4
    jmp .next_cluster

; Чётный кластер (Оставляем младшие 12 бит)
.even_cluster:
    and ax, 0x0FFF
    jmp .next_cluster

; Ошибка 2: Плохой кластер
.bad_cluster:
    mov si, err_bad_cluster
    jmp .fail

; Ошибка 4: Цепочка кластеров повреждена (Петля или служебный кластер)
.broken_chain:
    mov si, err_broken_chain
    jmp .fail

; Обработка следующего кластера
.next_cluster:
    ; Проверка на конец файла
    cmp ax, CHAIN_END
    jae .done

    ; Проверка на Bad Cluster
    cmp ax, BAD_CLUSTER
    je .bad_cluster

    ; Кластеры 0 и 1 служебные, их быть не может...
    cmp ax, 2
    jb .broken_chain

    ; Ограничение длины цепочки (Вдруг поймали петлю)
    inc word [chain_length]
    cmp word [chain_length], FAT12_MAX_CHAIN
    ja .broken_chain

    ; Обновляем номер текущего кластера, продолжая чтение
    mov [file_cluster], ax
    jmp .load_loop

; Неудачное завершение (si содержит сообщение об ошибке)
.fail:
    stc
    jmp .return

; Удачное завершение (с переносом строки после "кубиков" прогресса)
.done:
    call print_new_line_if_needed
    xor si, si
    clc

.return:
    pop es
    pop di
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; > Проверка пересечения диапазона [start, end) с записями таблицы регионов
; Параметры:
;  - di: смещение адреса таблицы регионов
;  - ebx: линейный start, ecx: линейный end
; Вывод:
;  - CF (Carry Flag): 0 (Нет пересечений), 1 (Найдено пересечение)
check_table:
    push eax
    push edx
    push di

.scan:
    ; Проверяем текущую запись о регионе (терминатор -1 -> выход)
    mov eax, [di]      ; Начало региона из таблицы (a)
    cmp eax, -1
    je .ok
    mov edx, [di + 4]  ; Конец региона из таблицы (b)

    ; Пересечение [start, end) и [a, b): start < b && a < end
    cmp ebx, edx
    jae .next
    cmp eax, ecx
    jb .hit

.next:
    ; Прибавляем размер записи (8 байт) и переходим к след.
    add di, 8
    jmp .scan

.hit:
    stc
    jmp .return

.ok:
    clc

.return:
    pop di
    pop edx
    pop eax
    ret

; Переменные модуля
filename_offset:    dw 0
dest_segment:       dw 0
dest_offset:        dw 0
extra_table_offset: dw 0
file_cluster:       dw 0
file_size:          dd 0
chain_length:       dw 0

; Сообщения об ошибках
err_not_found:    db '[!] E1: File not found!', 0
err_bad_cluster:  db '[!] E2: Bad cluster found!', 0
err_guard:        db '[!] E3: Destination overlaps a protected region!', 0
err_broken_chain: db '[!] E4: Broken FAT chain!', 0

; Core-таблица защищённых регионов памяти: критичные регионы
guard_core:
    dd 0x00000, 0x00500                   ; IVT + BIOS Data Area
    dd 0x00500, 0x07C00                   ; Временный буфер Root dir / FAT
    dd 0x07C00, 0x07E00                   ; Bootix (Stage 1)
    dd INITRIX_LOAD_SEGMENT * 16, 0xA000  ; Initrix (Stage 2)
    dd 0xA0000, 0x100000                  ; Видеопамять + ROM
    dd -1                                 ; Терминатор

; Extra-таблица защищённых регионов памяти: kernel16
guard_kernel16:
    dd KERNEL_LOAD_SEGMENT * 16, 0x20000  ; Kernel16 (64 КБ зарезервировано)
    dd -1                                 ; Терминатор
