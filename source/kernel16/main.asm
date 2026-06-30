; © Realix > Kernel
; (13.06.26) v0.06
; ================

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

; > Установка ядра
kernel_start:
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
    mov si, cli_hint
	call print

main:
    call run_cli

.halt:
    cli
    hlt
    jmp $

; Подключение модулей
%include 'kernel16/io/print.asm'
%include 'kernel16/io/print_ctrl.asm'
%include 'kernel16/io/print_reg.asm'
%include 'kernel16/shell/cli.asm'
%include 'kernel16/shell/commands.asm'
%include 'drivers/network/rtl8139.asm'
%include 'bios-api/memory/high.asm'
%include 'bios-api/memory/low.asm'

; Сообщения и строки
msg_start_kernel: db '[+] Starting kernel.', ENTER, 0
msg_enter_os:     db 'Press any key to continue.', 0
cli_title:        db 'Welcome, Realix (Real Mode with NASM kernel)...', ENTER, 0
cli_hint:         db 'Type "help" for list of commands.', ENTER, ENTER, 0