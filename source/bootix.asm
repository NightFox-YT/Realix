; Realix > Bootix
; (C) v0.03 | 19.08.25
; ===============

; Настройка компиляции
org 0x7C00
bits 16

; Символы
%define ENTER 0x0D, 0x0A

; [FAT12] Настройка (48 байт)
jmp short start
nop

bpb_oem:                   db 'MSWIN4.1' ; Идентификатор OEM (8 байт)
bpb_bytes_per_sector:      dw 512        ; Кол-во байт на сектор (Floppy: 512)
bpb_sectors_per_cluster:   db 1          ; Кол-во секторов на кластер
bpb_reserved_sectors:      dw 1          ; Кол-во зарезервированных секторов (Сектор загрузчика)
bpb_fats:                  db 2          ; Кол-во FAT таблиц
bpb_dir_entries:           dw 0x0E0      ; Кол-во записей корневого каталога
bpb_total_sectors:         dw 2880       ; Кол-во секторов (2880 * 512 = 1.44 мб)
bpb_media_descriptor_type: db 0x0F0      ; Тип диска (F0 - 3.5" floppy disk)
bpb_sectors_per_fat:       dw 9          ; Кол-во секторов на FAT таблицу
bpb_sectors_per_track:     dw 18         ; Кол-во секторов на дорожку
bpb_heads:                 dw 2          ; Кол-во голов
bpb_hidden_sectors:        dd 0          ; Кол-во скрытых секторов
bpb_large_sectors:         dd 0          ; Кол-во секторов, следующих дальше 65535 сектора

; Дополнительные параметры (extended boot record)
ebr_drive_number:          db 0                  ; Номер диска (0x00 floppy / 0x80 hdd)
                           db 0                  ; Зарезервировано
ebr_signature:             db 0x29               ; Подпись (0x28 или 0x29)
ebr_volume_id:             db 23h, 07h, 20h, 25h ; Серийный номер (Произвольный)
ebr_volume_label:          db '  Realix   '      ; Название тома (11 байт)
ebr_system_id:             db 'FAT12   '         ; Тип файловой системы (FAT12, FAT16... / 8 байт)

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

    jmp main

; Основной код
main:
    ; Чтение информации с диска
    mov ax, 1      ; LBA
    mov cl, 3      ; Кол-во секторов для чтения
    mov bx, 0x7E00 ; Адрес данных для записи (после загрузчика)
    call disk_read

    mov si, msg_welcome ; "Приветствие"
    call print

; Остановка CPU
.halt:
    cli
    hlt

; Обработчик ошибок
error_handler:
    mov ah, 0    ; Режим ожидания нажатия
    int 0x16
    jmp 0FFFFh:0 ; Переход в начало BIOS для перезагрузки

; [Kernel] Print, print_reg
%include 'print.asm'
%include 'print_reg.asm'

; [Disk] Read, lba_to_chs
%include 'read.asm'
%include 'lba_to_chs.asm'

; Сообщения
msg_welcome:      db 'Welcome, Realix v0.03.', ENTER, 0
msg_read_success: db '[LOG] Read ', 0, " sectors, LBA: ", 0, ENTER, 0

err_read_failed:  db '[!] Read from disk failed!', ENTER, 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55