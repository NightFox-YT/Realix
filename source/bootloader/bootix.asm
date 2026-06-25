; © Realix > Bootix
; (14.06.26) v0.06
; ================

; Настройка компиляции
bits 16
org 0x7C00

; Основные константы
%include 'shared/config.asm'

; Настройка FAT12 (48 байт)
jmp short start
nop

bpb_oem:                 db 'MSWIN4.1'  ; OEM (8 байт)
bpb_bytes_per_sector:    dw 512         ; Байт на сектор
bpb_sectors_per_cluster: db 1           ; Секторов на кластер
bpb_reserved_sectors:    dw 1           ; Кол-во зарезервированных секторов
bpb_fat_count:           db 2           ; Кол-во FAT таблиц
bpb_dir_entries:         dw 0x0E0       ; Кол-во записей в корневом каталоге
bpb_total_sectors:       dw 2880        ; Кол-во секторов (2880 * 512 = 1.44 мб)
bpb_media_type:          db 0x0F0       ; Тип диска (F0 - 3.5" floppy disk)
bpb_sectors_per_fat:     dw 9           ; Секторов на FAT таблицу
bpb_sectors_per_track:   dw 18          ; Секторов на дорожку
bpb_heads:               dw 2           ; Кол-во голов
bpb_hidden_sectors:      dd 0           ; Кол-во скрытых секторов
bpb_large_sectors:       dd 0           ; Кол-во секторов свыше 65535

; Дополнительные параметры (extended boot record)
ebr_drive_number: db 0                  ; Номер диска (0x00 floppy / 0x80 hdd)
                  db 0                  ; Зарезервировано
ebr_signature:    db 29h                ; Подпись (28h или 29h)
ebr_volume_id:    db 52h, 45h, 41h, 4Ch ; Серийный номер (Произвольный)
ebr_volume_label: db 'Realix     '      ; Название тома (11 байт)
ebr_system_id:    db 'FAT12   '         ; Тип файловой системы (8 байт)


start:
    ; Настройка сегментных регистров (Напрямую настроить нельзя)
    xor ax, ax
    mov ds, ax
    mov es, ax

    ; Настройка стека
    mov ss, ax
    mov sp, 0x7C00

    ; Обновление номера диска (BIOS устанавливает его в dl)
    mov [ebr_drive_number], dl

    ; Сброс сегмента кода `cs` дальним переходом
    jmp 0:main


main:
    ; Инициализация драйвера диска
    call disk_init
    mov [bpb_sectors_per_track], cx
    mov [bpb_heads], dh

    ; Вычисление LBA корневого каталога
    ; > LBA = fats * sectors_per_fat + reserved
    xor ah, ah
    mov al, [bpb_fat_count]
    mul word [bpb_sectors_per_fat]
    add ax, [bpb_reserved_sectors]

    ; *Сохраняем LBA корневого каталога
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

    ; Чтение корневого каталога
    mov cl, al                  ; Кол-во секторов - размер каталога
    pop ax                      ; *Восстанавливаем сохранённый LBA каталога
    mov dl, [ebr_drive_number]  ; Номер диска
    mov bx, 0x0500              ; Адрес записи
    call disk_read

    ; Подготовка к поиску файла
    xor bx, bx      ; Кол-во пройденных записей корневого каталога
    mov di, 0x0500  ; Адрес текущей записи корневого каталога

.search_initrix:
    ; Подготовка к сравнению названий (до 11 символов)
    mov si, file_initrix_bin
    mov cx, 11

    ; Сравнение по символу названия файлов, сохраняя адрес записи
    ; > si:di++ до cx == 0
    push di
    repe cmpsb
    pop di
    je .found_initrix

    ; Переход к следующей записи
    add di, 32                ; Увеличиваем адрес на размер записи (32 байта)
    inc bx                    ; Увеличиваем индекс записи
    cmp bx, [bpb_dir_entries]
    jl .search_initrix        ; Если не вышли за предел, продолжаем поиск

    ; Выход за предел, => файла второго этапа загрузчика нет
    mov si, err_initrix_not_found
    jmp error_handler

.found_initrix:
    ; Обновление номера кластера (di - адрес записи корневого каталога)
    mov ax, [di + 26]  ; Поле первого кластера (Смещение 26 байтов)
    push ax            ; *Сохраняем номер кластера

    ; Чтение FAT таблицы
    mov ax, [bpb_reserved_sectors]  ; LBA
    mov cl, [bpb_sectors_per_fat]   ; Кол-во секторов - размер FAT
    mov dl, [ebr_drive_number]      ; Номер диска
    mov bx, 0x0500                  ; Адрес записи
    call disk_read

    ; Установка сегмента и смещения для чтения initrix
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
    xor ch, ch
    mov cl, [bpb_sectors_per_cluster]
    mul cx
    add ax, [data_lba]

    ; Чтение следующего кластера (cl уже содержит нужное кол-во секторов)
    mov dl, [ebr_drive_number]
    call disk_read

    ; Увеличиваем адрес смещения initrix на кол-во прочитанных байт
    ; ❗️ NOTE: Initrix должен быть <=64 КБ, т.к. мы не обновляем сегмент ES
    xor ah, ah
    mov al, [bpb_sectors_per_cluster]
    mul word [bpb_bytes_per_sector]
    add bx, ax

    ; Вычисление смещения след. кластера в таблице FAT
    ; (ax - индекс записи, dx - Cluster % 2)
    pop ax     ; *Восстанавливаем номер кластера
    mov cx, 3
    mul cx
    mov cx, 2
    div cx

    ; Считывание записи из таблицы FAT
    mov si, 0x0500
    add si, ax
    mov ax, [ds:si]

    ; Проверка чётности кластера
    or dx, dx
    jz .even_cluster

.odd_cluster:
    ; Нечётный кластер - оставляем старшие 12 бит
    shr ax, 4
    jmp .next_cluster_after

.even_cluster:
    ; Чётный кластер - оставляем младшие 12 бит
    and ax, 0x0FFF

.next_cluster_after: 
    ; Проверка на конец файла
    cmp ax, 0x0FF8
    jae .read_initrix_finish

    ; *Сохраняем номер кластера для след. итерации, продолжая чтение
    push ax
    jmp .load_initrix_loop

.read_initrix_finish: 
    ; Настройка сегментных регистров под initrix
    mov ax, INITRIX_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    mov dl, [ebr_drive_number]

    ; Переход к initrix
    jmp INITRIX_LOAD_SEGMENT:INITRIX_LOAD_OFFSET


; > Обработчик ошибок
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    call print

    ; Ожидание нажатия
    mov ah, 0
    int 0x16

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0

; Подключение модулей
%include 'kernel16/io/print.asm'
%include 'bios-api/disk/read.asm'

; Сообщения
err_initrix_not_found: db '[!] No Initrix!', 0

; Переменные (Для чтения initrix)
file_initrix_bin: db 'INITRIX BIN'
data_lba:         dw 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55