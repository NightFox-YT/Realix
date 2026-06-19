; © Realix > FAT12 Initialization
; (13.06.26) v0.06
; ================

; Защита от повторного включения
%ifndef FAT12_INIT
%define FAT12_INIT

; > Инициализация параметров FAT12 (Вызывается 1 раз при подключении)
fat12_init:
    push ax
    push bx
    push cx
    push dx
    push es

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

.calculations:
    ; Валидация BPB перед вычислениями (Защита от переполнения)
    cmp word [bytes_per_sector], 512
    jne read_error
    cmp word [dir_entries], 224
    ja read_error

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
    shl ax, 5
    xor dx, dx
    div word [bytes_per_sector]

    ; Округление размера корневого каталога до целого вверх
    or dx, dx
    jz .save_other_fat_params
    inc ax

.save_other_fat_params:
    ; Обновление переменных
    mov [root_dir_size], ax
    add ax, [root_dir_lba]
    mov [data_lba], ax

.done:
    pop es
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Параметры FAT12
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

%endif
