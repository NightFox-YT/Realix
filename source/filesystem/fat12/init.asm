; © Realix > FAT12: Initialization
; (13.08.26) v0.12
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
    jz .save_params
    inc ax

.save_params:
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

%endif