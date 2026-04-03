; © Realix > Bootix
; (03.04.26) v0.04
; ================

; Настройка компиляции
bits 16
org 0x7C00

; Символы
%define ENTER 0x0D, 0x0A

; Настройка FAT12 (48 байт)
jmp short start
nop

bpb_oem:                 db 'MSWIN4.1' ; OEM (8 байт)
bpb_bytes_per_sector:    dw 512        ; Байт на сектор (Floppy: 512)
bpb_sectors_per_cluster: db 1          ; Секторов на кластер
bpb_reserved_sectors:    dw 1          ; Зарезервированных секторов
bpb_fat_count:           db 2          ; Кол-во FAT таблиц
bpb_dir_entries:         dw 0x0E0      ; Кол-во записей в корневом каталоге
bpb_total_sectors:       dw 2880       ; Кол-во секторов (2880 * 512 = 1.44 мб)
bpb_media_type:          db 0x0F0      ; Тип диска (F0 - 3.5" floppy disk)
bpb_sectors_per_fat:     dw 9          ; Секторов на FAT таблицу
bpb_sectors_per_track:   dw 18         ; Секторов на дорожку
bpb_heads:               dw 2          ; Кол-во голов
bpb_hidden_sectors:      dd 0          ; Кол-во скрытых секторов
bpb_large_sectors:       dd 0          ; Кол-во секторов свыше 65535

; Дополнительные параметры (extended boot record)
ebr_drive_number: db 0                  ; Номер диска (0x00 floppy / 0x80 hdd)
                  db 0                  ; Зарезервировано
ebr_signature:    db 0x29               ; Подпись (0x28 или 0x29)
ebr_volume_id:    db 23h, 07h, 20h, 25h ; Серийный номер (Произвольный)
ebr_volume_label: db 'Realix     '      ; Название тома (11 байт)
ebr_system_id:    db 'FAT12   '         ; Тип файловой системы (8 байт)

; Запуск
start:
    ; Настройка сегментных регистров (Напрямую настроить нельзя)
    xor ax, ax
    mov ds, ax
    mov es, ax

    ; Настройка стека (Стек растёт вниз от адреса загрузки)
    mov ss, ax
    mov sp, 0x7C00

    ; Обновление номера диска (BIOS устанавливает его в dl)
    mov [ebr_drive_number], dl

    ; Настройка сегмента кода (cs) дальним переходом
    ; (BIOS может запуститься по адресу 07C0:0000 вместо 0000:7C00)
    jmp 0:main

; Основной код
main:
    ; Считывание параметров диска
    call disk_params
    mov [bpb_sectors_per_track], cx
    mov [bpb_heads], dh

    ; Вычисление LBA корневого каталога
    ; > LBA = sectors_per_fat * fats + reserved
    mov ax, [bpb_sectors_per_fat]
    mov bl, [bpb_fat_count]
    xor bh, bh
    mul bx
    add ax, [bpb_reserved_sectors]

    ; *Сохраняем LBA корневого каталога
    push ax
    mov cx, ax

    ; Вычисление размера корневого каталога
    ; > root_dir_size = (32 * number_of_entries) / bytes_per_sector
    mov ax, [bpb_dir_entries]
    shl ax, 5                       ; ax = number_of_entries * 32
    xor dx, dx
    div word [bpb_bytes_per_sector]

    ; Округление размера корневого каталога до целого вверх
    or dx, dx    
    jz .read_root_dir
    inc ax

; Чтение корневого каталога
.read_root_dir:
    ; Обновление переменной root_dir_end
    add cx, ax
    mov [root_dir_end], cx

    ; Чтение корневого каталога
    pop ax                            ; *Восстанавливаем LBA каталога
    mov cl, al                        ; Кол-во секторов - размер каталога
    mov dl, [ebr_drive_number]        ; Номер диска
    mov bx, 0x7E00                    ; Адрес данных для записи
    push word [bpb_sectors_per_track]
    push word [bpb_heads]
    call disk_read

    ; Подготовка к поиску файла
    xor bx, bx      ; Кол-во пройденных записей корневого каталога
    mov di, 0x7E00  ; Адрес текущей записи корневого каталога

; Поиск initrix
.search_initrix:
    ; Подготовка к сравнению названий
    mov si, file_initrix_bin
    mov cx, 11                ; Сравниваем названия до 11 символов

    ; Сравниваем по символу название файлов, сохраняя адрес записи
    ; > si:di++ до cx == 0
    push di
    repe cmpsb
    pop di
    je .found_initrix

    ; Переход к след. записи
    add di, 32                ; Увеличиваем адрес на размер записи (32 байта)
    inc bx                    ; Увеличиваем индекс записи
    cmp bx, [bpb_dir_entries]
    jl .search_initrix        ; Если не вышли за предел, продолжаем поиск

    ; Вышли за предел, => файла второго этапа загрузчика нет
    mov si, err_initrix_not_found
    jmp error_handler

; Найден initrix
.found_initrix:
    ; Обновление номера кластера
    ; (di - адрес нужной записи корневого каталога)
    mov ax, [di + 26]         ; Поле первого кластера (Смещение 26 байтов)
    mov [initrix_cluster], ax ; Обновляем переменную

    ; Чтение FAT таблицы
    mov ax, [bpb_reserved_sectors]     ; LBA
    mov cl, [bpb_sectors_per_fat]      ; Кол-во секторов
    mov dl, [ebr_drive_number]         ; Номер диска
    mov bx, 0x7E00                     ; Адрес данных для записи
    push word [bpb_sectors_per_track]
    push word [bpb_heads]
    call disk_read

    ; Обновление сегментов и смещения для initrix
    mov bx, INITRIX_LOAD_SEGMENT
    mov es, bx
    mov bx, INITRIX_LOAD_OFFSET

; Чтение initrix и обработка FAT цепочки
.load_initrix_loop:
    ; Чтение след. кластера (dl содержит номер диска)
    push word [bpb_sectors_per_track]
    push word [bpb_heads]
    mov cl, [bpb_sectors_per_cluster] ; Кол-во секторов

    ; Вычисление LBA кластера
    ; > LBA = (initrix_cluster - 2) * sectors_per_cluster + root_dir_end
    mov ax, [initrix_cluster]
    sub ax, 2
    mul cx
    add ax, [root_dir_end]
    call disk_read

    ; Увеличиваем адрес смещения initrix на кол-во прочитанных байт
    ; ! NOTE: Initrix должен быть <=64 КБ, т.к. мы не изменяем сегмент
    xor ah, ah
    mov al, [bpb_sectors_per_cluster]
    mul word [bpb_bytes_per_sector]
    add bx, ax

    ; Вычисляем LBA следующего кластера
    ; > ax = Индекс записи, dx = Cluster % 2
    mov ax, [initrix_cluster]
    mov cx, 3
    mul cx
    mov cx, 2
    div cx

    ; Считывание записи из таблицы FAT по индексу (ax)
    mov si, 0x7E00
    add si, ax
    mov ax, [ds:si]

    ; Проверка чётности кластера
    or dx, dx
    jz .even_cluster

; Нечётный кластер (Оставляем старшие 12 бит)
.odd_cluster:
    shr ax, 4
    jmp .next_cluster_after

; Чётный кластер (Оставляем младшие 12 бит)
.even_cluster:
    and ax, 0x0FFF

; Обработка следующего кластера
.next_cluster_after: 
    ; Проверка на конец файла
    cmp ax, 0x0FF8
    jae .read_initrix_finish

    ; Обновляем номер текущего кластера, продолжая чтение
    mov [initrix_cluster], ax
    jmp .load_initrix_loop

; Заканчиваем чтение файла
.read_initrix_finish: 
    ; Настройка сегментов и регистров для initrix
    mov ax, INITRIX_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    mov dl, [ebr_drive_number]

    ; Переход к initrix...
    jmp INITRIX_LOAD_SEGMENT:INITRIX_LOAD_OFFSET

; > Обработчик ошибок
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    ; Вывод сообщения и ожидание нажатия
    call print
    mov ah, 0
    int 0x16

    ; Переход в вектор сброса BIOS
    jmp 0xFFFF:0

; Подключение модулей
%include 'kernel/print.asm'
%include 'disk/read.asm'

; Сообщения
err_initrix_not_found: db 'No Initrix!', 0

; Переменные (Для чтения второго этапа загрузчика)
file_initrix_bin: db 'INITRIX BIN'
initrix_cluster:  dw 0
root_dir_end:     dw 0

INITRIX_LOAD_SEGMENT equ 0x2000
INITRIX_LOAD_OFFSET  equ 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55