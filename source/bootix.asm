; Realix > Bootix
; (C) v0.04 | 25.08.25
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

    ; Настройка сегмента кода (cs) дальним переходом
    ; (BIOS может запустить загрузчик в 07C0:0000 вместо 0000:7C00)
    jmp 0:main

; Основной код
main:
    mov si, msg_booting ; "Загрузка"
    call print

    ; Считывание параметров диска (Не полагаемся на данные отформатированного диска)
    push es
    mov ah, 08h   ; Режим получения параметров диска
    int 0x13
    jc read_error ; Carry flag != 0, => Диск сломался...
    pop es

    and cl, 63                      ; Убираем верхние 2 бита в cl
    xor ch, ch
    mov [bpb_sectors_per_track], cx ; Обновляем кол-во секторов на дорожку

    inc dh              ; Увеличиваем dh (BIOS выводит кол-во голов минус 1)
    mov [bpb_heads], dh ; Обновляем кол-во голов

    jmp read_file ; Считываем файл второго этапа загрузчика

; Остановка CPU
.halt:
    cli
    hlt

; Обработчик ошибок
error_handler:
    mov ah, 0    ; Режим ожидания нажатия
    int 0x16
    jmp 0FFFFh:0 ; Переход в начало BIOS для перезагрузки

; [Kernel] Print
%include 'print.asm'

; [Disk] Read, read file, lba_to_chs
%include 'read.asm'
%include 'read_file.asm'
%include 'lba_to_chs.asm'

; Сообщения
msg_booting:           db 'Booting Realix.', 0
err_read_failed:       db 'Read failed!', 0
err_initrix_not_found: db 'No Initrix!', 0

; Переменные (Для чтение второго этапа загрузчика)
file_name: db 'INITRIX BIN'
FILE_LOAD_SEGMENT equ 0x2000
FILE_LOAD_OFFSET  equ 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55