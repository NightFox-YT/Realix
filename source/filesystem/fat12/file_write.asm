; © Realix > FAT12: File Write
; copyright by vv1ra
; (04.11.26) v0.1
; ================
; ❗️ Зависимости: bios-api/disk/write.asm, bios-api/disk/read.asm,
;                 filesystem/fat12/init.asm
; ❗️ Раскладка буферов рассчитана на дискету 1.44 МБ (BPB: spf=9, dir=224)
; ❗️ Условные прыжки к дальним меткам: только через инверсию + jmp
;    (Стабильность NASM-оптимизатора, см. file_load.asm)
; Инвариант модуля: es = 0 для доступа к буферам (root dir, FAT, список цепочки),
;                   es также используется как рабочий сегмент для дисковых вызовов
;                   (сохраняется/восстанавливается вокруг операций с диском)

; ❗️ Требуется инициализация FAT12 через `fat12_init`
%include "filesystem/fat12/init.asm"

; Модуль записи на диск (bios_disk_write)
%include "bios-api/disk/write.asm"

; Основные константы
%include 'shared/config.asm'

; Настройка DiskAPI: Write (Если не настроен)
%ifndef DISK_WRITE
%define DISK_WRITE bios_disk_write
%endif

; Защита от повторного включения
%ifndef FAT12_FILE_WRITE
%define FAT12_FILE_WRITE

; Раскладка записи корневого каталога FAT12 (32 байта)
; ❗️ Те же смещения используются в filesystem/fat12/file_load.asm
DIRENT_LEN      equ 32    ; Размер одной записи
DIRENT_NAME     equ 0     ; Имя файла (8 байт, формат 8.3)
DIRENT_ATTR     equ 11    ; Атрибуты файла (byte)
DIRENT_CLUSTER  equ 26    ; Номер первого кластера (word)
DIRENT_FILESIZE equ 28    ; Размер файла (dword)
DIRENT_FREE     equ 0x00  ; Первый байт имени: запись свободна (конец каталога)
DIRENT_DELETED  equ 0xE5  ; Первый байт имени: файл удалён
ATTR_ARCHIVE    equ 0x20  ; Атрибут "архивный" (Стандарт DOS для новых файлов)

; Адреса буферов модуля (Статическая раскладка памяти: es = 0)
; > FAT_BUFFER_ADDR (0x0500) - общий буфер FAT из shared/config
; > Root dir: за FAT-буфером с запасом (FAT дискеты 1.44МБ: 9 * 512 = 0x1200)
; > Список цепочки: за буфером root dir (Каталог 1.44МБ: 14 * 512 = 0x1C00)
WRITE_ROOT_BUFFER_ADDR equ 0x1E00  ; Буфер корневого каталога
WRITE_CHAIN_LIST_ADDR  equ 0x3B00  ; Список кластеров цепочки (word на кластер)

; Предельный размер файла по цепочке (Лимит списка цепочки)
WRITE_MAX_CLUSTERS equ 256

; Предел длины освобождаемой цепочки (Архитектурный потолок FAT12, от петель)
WRITE_MAX_CHAIN equ 4084


; > Сохранение файла на диск (Создание / перезапись записи в корне)
; Параметры:
;  - si: смещение адреса имени файла (11 символов, формат 8.3)
;  - ax: размер файла в байтах (0 - пустой файл, цепочка не создаётся)
;  - cx: сегмент источника данных
;  - bx: смещение источника данных
; Вывод:
; ❗️ Входной si при возврате не сохраняется
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка)
;  - si: смещение адреса сообщения об ошибке (0 - успех)
fat12_write:
    push ax
    push bx
    push cx
    push dx
    push di
    push es

    ; Сохраняем входные данные (До изменения ax!)
    mov [w_filename_offset], si
    mov [w_file_size], ax
    mov [w_src_segment], cx
    mov [w_src_offset], bx

    ; Настраиваем флаг направления и сегменты buffers
    cld
    xor ax, ax
    mov es, ax

    ; Вычисление размера кластера в байтах
    ; > cluster_bytes = spc * bytes_per_sector
    xor ax, ax
    mov al, byte [sectors_per_cluster]
    mul word [bytes_per_sector]
    mov [w_cluster_bytes], ax

    ; Вычисление количества кластеров файла
    ; > cluster_count = ceil(size / cluster_bytes)
    mov ax, [w_file_size]
    test ax, ax
    jz .no_clusters

    mov cx, [w_cluster_bytes]
    dec ax           ; size - 1 (Защита от лишнего кластера при кратном размере)
    xor dx, dx
    div cx
    inc ax

    ; Проверка предельного размера файла
    cmp ax, WRITE_MAX_CLUSTERS
    jbe .size_ok
    mov si, err_too_large
    jmp .fail

.size_ok:
    mov [w_cluster_count], ax
    jmp .have_clusters

.no_clusters:
    ; Пустой файл: Цепочки нет, первый кластер = 0
    mov word [w_cluster_count], 0
    mov word [w_first_cluster], 0

.have_clusters:
    ; Чтение корневого каталога в буфер модуля
    mov ax, [root_dir_lba]
    mov cl, byte [root_dir_size]
    mov bx, WRITE_ROOT_BUFFER_ADDR
    call DISK_READ

    ; Подготовка к поиску записи файла
    ; > Ищем точное совпадение имени (Перезапись), запоминая первую
    ;   свободную запись (0x00 / 0xE5) на случай отсутствия совпадений
    xor bx, bx               ; Счётчик пройденных записей
    mov di, WRITE_ROOT_BUFFER_ADDR
    mov word [w_match_di], 0
    mov word [w_free_di], 0

.search:
    ; Проверка первого байта имени записи
    cmp byte [es:di + DIRENT_NAME], DIRENT_FREE
    je .free_entry_found   ; 0x00 = конец каталога: свободен и дальше нет записей
    cmp byte [es:di + DIRENT_NAME], DIRENT_DELETED
    je .free_entry

    ; Сравнение имени по символам (до 11 символов)
    ; > ds:si (Имя) & fs:di (Буфер каталога)
    mov si, [w_filename_offset]
    mov cx, 11
    push di
    repe cmpsb
    pop di
    jne .search_next
    mov [w_match_di], di     ; Точное совпадение: Файл будет перезаписан
    jmp .check_free

.free_entry_found:
    ; 0x00 = конец каталога (FAT12): запись свободна, и все последующие тоже свободны.
    ; Запоминаем как свободный слот и прерываем поиск.
    cmp word [w_free_di], 0
    jne .have_entry
    mov [w_free_di], di
    jmp .have_entry

.free_entry:
    ; Запоминаем первую свободную запись (Продолжаем искать совпадения)
    cmp word [w_free_di], 0
    jne .search_next
    mov [w_free_di], di
    jmp .search_next

.search_next:
    ; Переход к следующей записи
    add di, DIRENT_LEN
    inc bx
    cmp bx, [dir_entries]
    jb .search

.check_free:
    ; Выбор целевой записи: Совпадение или свободный слот
    mov di, [w_match_di]
    or di, di
    jnz .have_entry

    mov di, [w_free_di]
    or di, di
    jnz .have_entry

    ; Ошибка E12: Свободных записей нет
    mov si, err_no_entry
    jmp .fail

.have_entry:
    ; Сохраняем целевую запись
    mov [w_dest_di], di

    ; Чтение FAT таблицы в буфер (Работаем с первой копией)
    mov ax, [reserved_sectors]
    mov cl, byte [sectors_per_fat]
    mov bx, FAT_BUFFER_ADDR
    call DISK_READ

    ; Освобождение старой цепочки файла
    ; (Только для перезаписи существующего файла: у свободной записи
    ;  поле кластера может содержать мусор от удалённого файла)
    mov word [w_chain_length], 0
    cmp word [w_match_di], 0
    je .alloc                ; Новый файл: Освобождать нечего

    mov ax, [es:di + DIRENT_CLUSTER]
    cmp ax, 2
    jb .alloc

    ; Запоминаем старый первый кластер (Для освобождения цепочки)
    mov [w_old_cluster], ax

.free_loop:
    ; Обнуление записи текущего кластера в FAT буфере
    ; > ax: номер кластера, bx: новое значение
    mov bx, 0
    call fat12_write_entry

    ; Фиксируем длину освобождённой цепочки (Защита от петли)
    inc word [w_chain_length]

    ; Чтение следующего кластера освобождённой цепочки
    mov ax, [w_old_cluster]
    call fat12_read_entry

    ; Проверка на конец цепочки (EOF / Bad / Служебные / Петля)
    cmp ax, BAD_CLUSTER
    je .free_end
    cmp ax, CHAIN_END
    jae .free_end
    cmp ax, 2
    jb .free_end
    cmp word [w_chain_length], WRITE_MAX_CHAIN
    jae .free_end

    ; Переход к следующему кластеру цепочки
    mov [w_old_cluster], ax
    jmp .free_loop

.free_end:
.alloc:
    ; Поиск свободных кластеров и формирование списка цепочки
    ; > Сканируем записи FAT в диапазоне кластеров [2, max_data_cluster]
    ; > В буфер собираем список номеров кластеров в порядке следования
    mov cx, [w_cluster_count]
    or cx, cx
    jnz .alloc_init
    jmp .data_write          ; Пустой файл: Цепочка не нужна

.alloc_init:
    mov word [w_clusters_left], cx
    mov di, WRITE_CHAIN_LIST_ADDR
    mov bx, 2                ; Позиция сканирования FAT (Нумерация с 2)

.alloc_scan:
    ; Если свободные кластеры кончились - диск полон
    cmp bx, [max_data_cluster]
    jbe .alloc_entry
    mov si, err_disk_full
    jmp .fail

.alloc_entry:
    ; Чтение записи FAT текущей позиции (ax: номер кластера)
    mov ax, bx
    call fat12_read_entry

    ; Проверка на свободный кластер (Значение 0)
    test ax, ax
    jz .alloc_take

    ; Занят: Переход к следующей записи FAT
    inc bx
    jmp .alloc_scan

.alloc_take:
    ; Добавляем кластер в список цепочки
    mov [es:di], bx
    add di, 2

    ; Уменьшаем счётчик оставшихся кластеров
    dec word [w_clusters_left]
    jz .alloc_done

    ; Выделение не завершено: Сканируем дальше
    inc bx
    jmp .alloc_scan

.alloc_done:
    ; Запоминаем первый кластер нового файла
    mov ax, [es:WRITE_CHAIN_LIST_ADDR]
    mov [w_first_cluster], ax

    ; Заполнение FAT цепочки: Каждой записи -> след. кластер (Последний -> EOF)
    mov cx, [w_cluster_count]
    mov di, WRITE_CHAIN_LIST_ADDR

.chain_loop:
    ; Текущий кластер
    mov ax, [es:di]

    ; Следующее значение записи: Последнее звено -> EOF
    mov bx, CHAIN_END
    cmp cx, 1
    jz .chain_store

    ; Есть след. кластер: Ссылаемся на него
    mov bx, [es:di + 2]

.chain_store:
    ; Запись значения в FAT буфер
    call fat12_write_entry

    ; Переход к следующему звену цепочки
    add di, 2
    dec cx
    jnz .chain_loop

.data_write:
    ; Запись данных файла по кластерам (Пропускается для пустого файла)
    mov cx, [w_cluster_count]
    or cx, cx
    jnz .data_init
    jmp .update_entry

.data_init:
    ; Установка сегмента источника данных (В es)
    ; > fs остаётся нулевым для доступа к списку цепочки
    mov ax, [w_src_segment]
    mov es, ax
    mov di, WRITE_CHAIN_LIST_ADDR

.data_loop:
    ; Вычисление LBA кластера
    ; > LBA = (cluster - 2) * spc + data_lba
    xor cx, cx
    mov cl, byte [sectors_per_cluster]
    push es                 ; Сохраняем ES (=es источник данных)
    xor ax, ax
    mov es, ax              ; ES = 0 для чтения буфера цепочки
    mov ax, [es:di]         ; Текущий кластер из списка цепочки
    pop es                  ; Восстанавливаем ES = источник данных
    sub ax, 2
    mul cx
    add ax, [data_lba]

    ; Адрес источника данных (cl - кол-во секторов)
    mov cl, byte [sectors_per_cluster]
    mov bx, [w_src_offset]

    ; Запись кластера на диск
    call DISK_WRITE
    jnc .data_advance

    ; Ошибка E11: Запись на диск не удалась
    mov si, err_write_failed
    jmp .fail

.data_advance:
    ; Увеличиваем адрес источника на размер кластера
    mov ax, [w_cluster_bytes]
    add bx, ax
    mov [w_src_offset], bx
    jnc .data_next

    ; Сдвигаем сегмент источника на 0x1000 параграфов (+64 КБ)
    mov ax, es
    add ax, 0x1000
    mov es, ax
    mov word [w_src_offset], 0

.data_next:
    ; Переход к следующему кластеру цепочки
    add di, 2
    dec cx
    jnz .data_loop

.write_fat:
    ; Возврат к сегменту 0 для записи FAT (Буфер в fs = 0)
    xor ax, ax
    mov es, ax

    ; Запись FAT таблицы на диск: Все копии последовательно
    ; > LBA копии i = reserved + i * sectors_per_fat
    ; > ch - индекс копии (cl используется как счётчик секторов)
    xor cx, cx

.fat_loop:
    ; Вычисление LBA текущей копии FAT
    mov ax, cx
    mul word [sectors_per_fat]
    add ax, [reserved_sectors]

    ; Запись копии FAT (cl - кол-во секторов)
    mov cl, byte [sectors_per_fat]
    mov bx, FAT_BUFFER_ADDR
    call DISK_WRITE
    jnc .fat_next

    ; Ошибка E11: Запись на диск не удалась
    mov si, err_write_failed
    jmp .fail

.fat_next:
    ; Переход к следующей копии FAT
    inc ch
    cmp ch, byte [fat_count]
    jb .fat_loop

.update_entry:
    ; Возврат к сегменту 0 для записи каталога
    xor ax, ax
    mov es, ax

    ; Копирование имени файла в запись каталога
    ; > ds:si (Имя) -> fs:di (Буфер каталога)
    mov di, [w_dest_di]
    mov si, [w_filename_offset]
    mov cx, 11
    rep movsb

    ; Установка атрибутов (Архивный файл)
    mov byte [es:di], ATTR_ARCHIVE

    ; Обновление полей записи: Кластер и размер
    mov di, [w_dest_di]
    mov ax, [w_first_cluster]
    mov [es:di + DIRENT_CLUSTER], ax
    mov ax, [w_file_size]
    mov [es:di + DIRENT_FILESIZE], ax
    mov word [es:di + DIRENT_FILESIZE + 2], 0

    ; Запись обновлённого корневого каталога на диск
    mov ax, [root_dir_lba]
    mov cl, byte [root_dir_size]
    mov bx, WRITE_ROOT_BUFFER_ADDR
    call DISK_WRITE
    jnc .done

    ; Ошибка E11: Запись на диск не удалась
    mov si, err_write_failed
    jmp .fail

; Удачное завершение
.done:
    xor si, si
    clc
    jmp .return

; Неудачное завершение (si содержит сообщение об ошибке)
.fail:
    stc

.return:
    pop es
    pop di
    pop dx
    pop cx
    pop bx
    pop ax
    ret


; > Чтение записи FAT таблицы по номеру кластера
; Параметры:
;  - ax: номер кластера
; Вывод:
;  - ax: значение записи (12 бит)
;  - dx: чётность кластера (0 - чётный, 1 - нечётный)
fat12_read_entry:
    push bx
    push cx
    push si
    push es

    ; Вычисление байтового смещения записи
    ; > offset = cluster * 3 / 2, dx - чётность кластера
    mov cx, 3
    mul cx
    dec cx     ; cx: 3 -> 2 (mul не трогает cx)
    div cx

    ; Считывание 16-битного слова из FAT буфера
    xor bx, bx
    mov es, bx
    mov si, FAT_BUFFER_ADDR
    add si, ax
    mov ax, [es:si]

    ; Проверка чётности кластера
    or dx, dx
    jz .even_cluster

; Нечётный кластер (Оставляем старшие 12 бит)
.odd_cluster:
    shr ax, 4
    jmp .return

; Чётный кластер (Оставляем младшие 12 бит)
.even_cluster:
    and ax, 0x0FFF

.return:
    pop es
    pop si
    pop cx
    pop bx
    ret


; > Запись значения в FAT таблицу по номеру кластера
; (Пакует 12-битную запись в FAT12 с учётом соседней записи)
; Параметры:
;  - ax: номер кластера
;  - bx: новое значение записи (12 бит)
; Вывод: ничего
fat12_write_entry:
    push ax
    push cx
    push dx
    push si
    push es

    ; Вычисление байтового смещения записи
    ; > offset = cluster * 3 / 2, dx - чётность кластера
    mov cx, 3
    mul cx
    dec cx     ; cx: 3 -> 2 (mul не трогает cx)
    div cx

    ; Сегмент FAT буфера (0: адрес)
    xor si, si
    mov es, si
    mov si, FAT_BUFFER_ADDR
    add si, ax

    ; Считывание текущей записи (Упаковка: слияние с соседним кластером)
    mov ax, [es:si]

    ; Проверка чётности кластера
    or dx, dx
    jz .even_cluster

; Нечётный кластер (Старшие 12 бит: сохраняем младшие 4 бита соседней записи)
.odd_cluster:
    and ax, 0x000F
    add bx, bx        ; shl bx, 1 (эквивалент shl bx, 4 на 8086)
    add bx, bx
    add bx, bx
    add bx, bx
    or ax, bx
    jmp .store

; Чётный кластер (Младшие 12 бит: сохраняем старшие 4 бита соседней записи)
.even_cluster:
    and ax, 0xF000
    and bx, 0x0FFF
    or ax, bx

.store:
    mov [es:si], ax

    pop es
    pop si
    pop dx
    pop cx
    pop ax
    ret


; Переменные модуля
w_filename_offset: dw 0
w_src_segment:     dw 0
w_src_offset:      dw 0
w_file_size:       dw 0
w_cluster_bytes:   dw 0
w_cluster_count:   dw 0
w_clusters_left:   dw 0
w_free_di:         dw 0
w_match_di:        dw 0
w_dest_di:         dw 0
w_old_cluster:     dw 0
w_first_cluster:   dw 0
w_chain_length:    dw 0

; Сообщения об ошибках
; > err_write_failed (E11) определён в bios-api/disk/write.asm
err_no_entry:     db '[!] E12: No free entries in root directory!', 0
err_disk_full:    db '[!] E13: Not enough free clusters!', 0
err_too_large:    db '[!] E14: File too large to write!', 0

%endif
