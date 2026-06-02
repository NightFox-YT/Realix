; © Realix > FAT12: Initialization
; (01.06.26) v0.05
; ================

; > Инициализация параметров FAT12 (При необходимости)
fat_init:
    push ax
    push bx
    push cx
    push dx
    push es

    ; Проверка на необходимость инициализации
    cmp byte [fat_initialized], 1
    je .done

.initialization:
    ; Читаем данные BPB из сектора Bootix
    xor ax, ax
    mov es, ax

    mov ax, [es:0x7C00 + 11]
    mov [bytes_per_sector], ax
    mov al, [es:0x7C00 + 13]
    mov [sectors_per_cluster], al
    mov ax, [es:0x7C00 + 14]
    mov [reserved_sectors], ax
    mov al, [es:0x7C00 + 16]
    mov [fat_count], al
    mov ax, [es:0x7C00 + 17]
    mov [dir_entries], ax
    mov ax, [es:0x7C00 + 22]
    mov [sectors_per_fat], ax
    mov ax, [es:0x7C00 + 24]
    mov [sectors_per_track], ax
    mov ax, [es:0x7C00 + 26]
    mov [heads], ax

    ; Вычисление LBA корневого каталога
    ; > LBA = sectors_per_fat * fats + reserved
    mov ax, [sectors_per_fat]
    xor bh, bh
    mov bl, [fat_count]
    mul bx
    add ax, [reserved_sectors]
    mov [root_dir_lba], ax

    ; Вычисление размера корневого каталога
    ; > root_dir_size = (number_of_entries * 32) / bytes_per_sector
    mov ax, [dir_entries]
    shl ax, 5             ; *32 (number_of_entries * 32)
    xor dx, dx
    div word [bytes_per_sector]

    ; Округление размера корневого каталога до целого вверх
    or dx, dx
    jz .save_fat_params
    inc ax

.save_fat_params:
    ; Обновление переменных
    mov [root_dir_size], ax
    add ax, [root_dir_lba]
    mov [data_lba], ax

    ; Обновление флага инициализации
    mov byte [fat_initialized], 1

.done:
    pop es
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Параметры FAT12
fat_initialized: db 0
root_dir_lba:    dw 0
root_dir_size:   dw 0
data_lba:        dw 0

; Параметры BPB
bytes_per_sector:    dw 0
sectors_per_cluster: db 0
reserved_sectors:    dw 0
fat_count:           db 0
dir_entries:         dw 0
sectors_per_fat:     dw 0
sectors_per_track:   dw 0
heads:               dw 0