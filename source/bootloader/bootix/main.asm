; © Realix > Bootix (Stage 1)
; (13.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/print (в режиме PRINT_MINIMAL),
;                 bios-api/keyboard (в режиме KEYBOARD_MINIMAL),
;                 bios-api/disk/read

; Настройка компиляции
bits 16
org 0x7C00

; Основные константы
%include 'shared/config.asm'

; Настройка DiskAPI
%define DISK_INIT bios_disk_init
%define DISK_READ bios_disk_read

; Настройка заголовков FAT12 (48 байт)
jmp short setup
nop

bpb_oem:                 db 'MSWIN4.1'  ; OEM (8 байт)
bpb_bytes_per_sector:    dw 512         ; Байт на сектор
bpb_sectors_per_cluster: db 1           ; Секторов на кластер
bpb_reserved_sectors:    dw 1           ; Кол-во зарезервированных секторов
bpb_fat_count:           db 2           ; Кол-во FAT таблиц
bpb_dir_entries:         dw 0x0E0       ; Кол-во записей в корневом каталоге
bpb_total_sectors:       dw 2880        ; Кол-во секторов (2880 * 512 = 1.44 мб)
bpb_media_type:          db 0xF0        ; Тип диска (F0 - 3.5" floppy disk)
bpb_sectors_per_fat:     dw 9           ; Секторов на FAT таблицу
bpb_sectors_per_track:   dw 18          ; Секторов на дорожку
bpb_heads:               dw 2           ; Кол-во голов на диске
bpb_hidden_sectors:      dd 0           ; Кол-во скрытых секторов
bpb_large_sectors:       dd 0           ; Кол-во секторов (если свыше 65535)

; Дополнительные параметры (extended boot record)
ebr_drive_number: db 0                  ; Номер диска (0x00 floppy / 0x80 hdd)
                  db 0                  ; Зарезервировано (Флаги для Windows NT)
ebr_signature:    db 29h                ; Подпись (28h / 29h)
ebr_volume_id:    db 52h, 45h, 41h, 4Ch ; Серийный номер (Произвольный)
ebr_volume_label: db 'Realix     '      ; Название тома (11 байт)
ebr_system_id:    db 'FAT12   '         ; Тип файловой системы (8 байт)


; > Инициализация окружения
setup:
    ; Отключаем прерывания во время настройки стека и сегментов
    cli

    ; Настройка сегментных регистров (Напрямую настроить нельзя)
    xor ax, ax
    mov ds, ax
    mov es, ax

    ; Настройка стека
    mov ss, ax
    mov sp, 0x7C00

    ; Обновление номера диска (BIOS устанавливает его в dl)
    mov [ebr_drive_number], dl

    ; Сброс сегмента кода (cs) дальним переходом и включение прерываний
    sti
    jmp 0:main


; > Основной код
main:
    ; Инициализация драйвера диска (dl содержит номер диска)
    call DISK_INIT
    mov [bpb_sectors_per_track], cx
    mov [bpb_heads], dh

    ; > Далее загрузка 2-ого этапа загрузчика (Initrix.bin)
    ; ❗️ Свой мин. обход FAT12 вместо filesystem/fat12/file_load,
    ;    чтобы 1-ый этап влез в 512 байт (Дублирование намеренное)

    ; Вычисление LBA корневого каталога
    ; > LBA = fats * sectors_per_fat + reserved
    movzx ax, byte [bpb_fat_count]
    mul word [bpb_sectors_per_fat]
    add ax, [bpb_reserved_sectors]

    ; *Сохраняем LBA корневого каталога (root_dir_lba)
    push ax
    mov cx, ax

    ; Вычисление размера корневого каталога
    ; > root_dir_size = (number_of_entries * 32) / bytes_per_sector
    mov ax, [bpb_dir_entries]
    shl ax, 5                  ; *32 (number_of_entries * 32)
    xor dx, dx
    div word [bpb_bytes_per_sector]

    ; Округление размера корневого каталога до целого вверх
    or dx, dx
    jz .read_root_dir
    inc ax

.read_root_dir:
    ; Обновление переменной data_lba (root_dir_lba + root_dir_size)
    add cx, ax
    mov [data_lba], cx

    ; Чтение корневого каталога в память
    mov cl, al               ; Кол-во секторов - размер каталога
    pop ax                   ; *Восстанавливаем LBA каталога
    mov bx, FAT_BUFFER_ADDR  ; Адрес записи FAT буфера
    call DISK_READ

    ; Подготовка к поиску файла в корневом каталоге
    xor bx, bx               ; Счётчик пройденных записей
    mov di, FAT_BUFFER_ADDR  ; Смещение текущей записи

.search_initrix:
    ; Подготовка к сравнению названий (до 11 символов)
    mov si, file_initrix_bin
    mov cx, 11

    ; Сравнение по символу названия файлов, сохраняя смещение записи
    ; > ds:si & es:di; si++, di++ до cx == 0
    push di
    repe cmpsb
    pop di
    je .found_initrix

    ; Переход к следующей записи
    add di, 32                ; Увеличиваем смещение на размер записи (32 байта)
    inc bx                    ; Увеличиваем индекс записи
    cmp bx, [bpb_dir_entries]
    jb .search_initrix        ; Если не вышли за предел, продолжаем поиск

    ; Если "Выход за предел", => файла второго этапа загрузчика нет
    mov si, err_initrix_not_found
    jmp error_handler

.found_initrix:
    ; Обновление номера кластера (di - адрес записи корневого каталога)
    mov ax, [di + 26]  ; Поле первого кластера (Смещение 26 байтов)
    push ax            ; *Сохраняем номер кластера

    ; Чтение FAT таблицы в память
    mov ax, [bpb_reserved_sectors]       ; LBA
    mov cl, byte [bpb_sectors_per_fat]   ; Кол-во секторов - размер FAT
    mov bx, FAT_BUFFER_ADDR              ; Адрес записи FAT буфера
    call DISK_READ

    ; Установка сегмента и смещения для чтения Initrix
    mov bx, INITRIX_LOAD_SEGMENT
    mov es, bx
    mov bx, INITRIX_LOAD_OFFSET

.load_initrix_loop:
    ; *Восстанавливаем и сохраняем номер кластера
    pop ax
    push ax

    ; Вычисление LBA кластера
    ; > LBA = (initrix_cluster - 2) * sectors_per_cluster + data_lba
    sub ax, 2
    movzx cx, byte [bpb_sectors_per_cluster]
    mul cx
    add ax, [data_lba]

    ; Чтение следующего кластера (cl содержит кол-во секторов)
    call DISK_READ

    ; Увеличиваем адрес смещения Initrix на кол-во прочитанных байт
    movzx ax, byte [bpb_sectors_per_cluster]
    mul word [bpb_bytes_per_sector]
    add bx, ax
    jnc .load_initrix_continue

    ; Сдвигаем es на 0x1000 параграфов (+64 КБ)
    mov ax, es
    add ax, 0x1000
    mov es, ax
    xor bx, bx

.load_initrix_continue:
    ; Вычисляем байтовое смещение записи след. кластера
    ; > (cluster * 3 / 2), ax - смещение в байтах, dx - Cluster % 2
    pop ax     ; *Восстанавливаем номер кластера
    mov cx, 3
    mul cx
    dec cx     ; cx: 3 -> 2 (mul не трогает cx)
    div cx

    ; Считывание записи из таблицы FAT по индексу (ax)
    mov si, FAT_BUFFER_ADDR
    add si, ax
    mov ax, [si]

    ; Проверка чётности кластера
    or dx, dx
    jz .even_cluster

.odd_cluster:
    ; Нечётный кластер - оставляем старшие 12 бит
    shr ax, 4
    jmp .next_cluster

.even_cluster:
    ; Чётный кластер - оставляем младшие 12 бит
    and ax, 0x0FFF

.next_cluster:
    ; Проверка на конец файла
    cmp ax, CHAIN_END
    jae .initrix_load_finish

    ; *Сохраняем номер кластера для след. итерации, продолжая чтение
    push ax
    jmp .load_initrix_loop

.initrix_load_finish:
    ; Передача номера диска
    mov dl, [ebr_drive_number]
    
    ; Настройка сегментных регистров под Initrix
    mov ax, INITRIX_LOAD_SEGMENT
    mov ds, ax
    mov es, ax

    ; Переход к Initrix
    jmp INITRIX_LOAD_SEGMENT:INITRIX_LOAD_OFFSET


; > Обработчик критических ошибок (Имитация синего экрана)
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    ; Вывод сообщения и ожидание нажатия
    call print
    call wait_key

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0


; Подключение модулей
%define PRINT_MINIMAL
%include 'bios-api/io/print.asm'
%define KEYBOARD_MINIMAL
%include 'bios-api/keyboard.asm'
%include 'bios-api/disk/read.asm'

; Сообщения
err_initrix_not_found: db '[#] No Initrix!', 0

; Переменные (Для чтения initrix)
file_initrix_bin: db 'INITRIX BIN'
data_lba:         dw 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55