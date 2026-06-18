; © Realix > Kernel
; (13.06.26) v0.06
; ================

; Настройка компиляции
bits 16

global _start

; Основные константы
%include 'config.asm'
section .text
; > Установка ядра
_start:
	mov si, msg_start_kernel
	call print

    call print_new_line

	mov si, msg_enter_os
	call print

    ; Ожидание нажатия
	mov ah, 0x0
    int 0x16

    ; Настройка консоли
    call cmd_cls
	mov si, cli_title
	call print

main:
    call run_cli

.halt:
    cli
    hlt
    jmp $

; Подключение модулей
%include 'kernel16/io/print.asm'
%include 'kernel16/io/print_nl.asm'
%include 'kernel16/io/print_reg.asm'
%include 'kernel16/shell/cli.asm'
%include 'kernel16/shell/commands.asm'
%include 'bios-api/memory/get_free.asm'
%include 'bios-api/memory/get_lower.asm'
%include 'bios-api/memory/get_map.asm'

; Сообщения и строки
msg_start_kernel: db '[+] Starting kernel.', ENTER, 0
msg_enter_os:     db 'Welcome, press any key to continue.', 0
cli_title:            db 'Realix v0.06 / (C) NightFox developer', ENTER, ENTER, 0