; © Realix > Kernel
; (05.04.26) v0.06
; ================

; Настройка компиляции
bits 16
org 0x0

; Символы
%include 'symbols.inc'

; Установка ядра
setup_kernel:
	mov si, msg_start_kernel  ; "Включение ядра"
	call print

	mov si, msg_enter_os  ; "Нажмите кнопку, чтобы продолжить"
	call print

    ; Ожидание нажатия
	mov ah, 0x0
    int 0x16

    ; Очистка экрана с установкой курсора в начало
	mov ah, 0x00
    mov al, 0x03
    int 0x10

	mov si, title     ; "Заголовок консоли"
	call print

main:
    ; "Realix@User >> "
	mov si, msg_input
	call print

    ; Обработка ввода
	call input

	; Переход на новую строку
	mov si, new_line
	call print

    jmp main

; Подключение модулей
%include 'kernel/print.asm'
%include 'kernel/input.asm'

; Сообщения
msg_start_kernel: db '[+] Starting kernel.', ENTER, 0
msg_enter_os:     db ENTER, 'Welcome, press any key to continue.', 0
title:            db 'Realix v0.06 / (C) NightFox developer', ENTER, ENTER, 0
msg_input:        db 'Realix@User >> ', 0

; Вспомогательные строки
new_line: db ENTER, 0