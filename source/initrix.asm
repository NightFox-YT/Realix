; © Realix > Initrix
; (05.04.26) v0.05
; ================

; Настройка компиляции
bits 16
org 0x0

; Символы
%define ENTER 0x0D, 0x0A

; Основной код
main:
    ; Получение номера диска, переданного из bootix
    mov [boot_drive], dl

    mov si, msg_init  ; "Инициализация"
    call print

    ; Получение объёма доступной "нижней" памяти (до 640 КБ)
    clc
    int 12h
    jc memory_error
    mov [memory_kb], ax

    ; Сохранение собранной информации (С сбросом сегмента на 0x0)
    push ds
    xor ax, ax
    mov ds, ax

    mov ax, [memory_kb]
    mov word [PCINFO_ADDR], ax      ; Размер нижней памяти (КБ)
    mov byte [PCINFO_ADDR + 2], dl  ; Номер загрузочного диска

    pop ds

    ; Очистка экрана
    mov ah, 00h
    mov al, 03h
    int 10h

    ; Экран загрузки (+короткий звук)
    mov si, char_beep
    call print

    mov si, str_title
    call print

    mov si, str_memory
    call print
    mov ax, [memory_kb]
    call print_reg
    mov si, str_kb
    call print

    mov si, double_new_line
    call print

    ; "Загрузка ядра..."
    mov si, msg_loading
    call print

    ; Загрузка ядра с диска
    mov si, kernel_filename
    mov ax, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    call load_file

    ; Настройка сегментов и регистров под kernel
    mov ax, KERNEL_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    mov di, PCINFO_ADDR

    ; Передача управления ядру
    jmp KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET

; Остановка CPU
halt:
    cli
    hlt
    jmp $

; Ошибка памяти
memory_error:
    mov si, err_memory
    jmp error_handler

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
%include 'kernel/print_dec.asm'
%include 'disk/read.asm'
%include 'disk/fat12.asm'

; Сообщения
msg_init:    db '[+] Initializing.', ENTER, 0
msg_loading: db '[+] Loading kernel.', ENTER, 0
err_memory:  db '[!] Memory error (int 12h)!', ENTER, 0

; Строки загрузочного экрана
str_title:       db '     Realix v0.05', ENTER, '(C) NightFox developer', ENTER, ENTER, 0
str_memory:      db 'Low Memory: ', 0
str_kb:          db ' KB / 640 KB', 0

; Вспомогательные строки
char_beep:       db 0x07, 0
new_line:        db ENTER, 0
double_new_line: db ENTER, ENTER, 0

; Переменные
kernel_filename: db 'KERNEL  BIN'
boot_drive:      db 0
memory_kb:       dw 0

KERNEL_LOAD_SEGMENT equ 0x1000
KERNEL_LOAD_OFFSET  equ 0x0000
PCINFO_ADDR equ 0x0500