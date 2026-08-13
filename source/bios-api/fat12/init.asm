; © Realix > FAT12: Initialization
; (27.07.26) v0.1
; ================
; ❗️ Требует запущенного Bootix с BPB по адресу 0x0:0x7C00

; Защита от повторного включения
%ifndef FAT12_INIT
%define FAT12_INIT

; Основные константы
%include 'shared/config.asm'

; > Инициализация параметров FAT12
; (Вызывается при переключении/инициализации диска)
fat12_init:
    push ax
    push bx
    push cx
    push dx
    push es

    ; Читаем данные BPB из сектора Bootix
    xor ax, ax
    mov es, ax

    mov ax, [es:BOOTIX_LOAD_ADDR + 11]
    mov [bytes_per_sector], ax
    mov al, [es:BOOTIX_LOAD_ADDR + 13]
    mov [sectors_per_cluster], al
    mov ax, [es:BOOTIX_LOAD_ADDR + 14]
    mov [reserved_sectors], ax
    mov al, [es:BOOTIX_LOAD_ADDR + 16]
    mov [fat_count], al
    mov ax, [es:BOOTIX_LOAD_ADDR + 17]
    mov [dir_entries], ax
    mov ax, [es:BOOTIX_LOAD_ADDR + 22]
    mov [sectors_per_fat], ax

.calculations:
    ; Вычисление LBA корневого каталога
    ; > LBA = sectors_per_fat * fats + reserved
    mov ax, [sectors_per_fat]
    movzx bx, byte [fat_count]
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

; Переменные FAT12
root_dir_lba:  dw 0
root_dir_size: dw 0
data_lba:      dw 0

; Переменные BPB
bytes_per_sector:    dw 0
sectors_per_cluster: db 0
reserved_sectors:    dw 0
fat_count:           db 0
dir_entries:         dw 0
sectors_per_fat:     dw 0

%endif