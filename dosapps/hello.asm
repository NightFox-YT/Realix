; © Realix > Sample MS-DOS .COM Application
; ø Copyright by @Ramix
; (08.09.26) v0.12 [Nightly]
; ================
; ❗️ Обычная программа для MS-DOS: собирается как плоский .COM (org 0x100),
;    использует только стандартный int 21h - должна работать и под настоящим DOS,
;    и под msexec kernel16-nightly. Демонстрирует функции 09h (строка), 01h (клавиша),
;    0Ah (буферизованный ввод), 4Ch (выход с кодом).

bits 16
org 0x100

start:
    ; AH=09h: печать строки, завершённой символом '$'
    mov ah, 0x09
    mov dx, msg_hello
    int 0x21

    ; AH=0Ah: буферизованный ввод строки (Имя)
    mov ah, 0x09
    mov dx, msg_prompt
    int 0x21

    mov dx, input_buffer
    mov ah, 0x0A
    int 0x21

    ; Перевод строки перед ответом
    mov ah, 0x09
    mov dx, msg_newline
    int 0x21

    ; AH=09h: печать приветствия
    mov ah, 0x09
    mov dx, msg_greet
    int 0x21

    ; Печать введённого имени (input_buffer+2, длина в input_buffer+1)
    mov si, input_buffer + 2
    movzx cx, byte [input_buffer + 1]
    test cx, cx
    jz .skip_name

.print_name_loop:
    mov dl, [si]
    mov ah, 0x02
    int 0x21
    inc si
    loop .print_name_loop

.skip_name:
    mov ah, 0x09
    mov dx, msg_bang
    int 0x21

    ; AH=01h: "Press any key..."
    mov ah, 0x09
    mov dx, msg_press_key
    int 0x21
    mov ah, 0x01
    int 0x21

    ; AH=4Ch: выход с кодом возврата 0
    mov ax, 0x4C00
    int 0x21

msg_hello:     db 13, 10, '[DOS .COM] Hello from a real MS-DOS-style program!', 13, 10, '$'
msg_prompt:    db 'What is your name? $'
msg_newline:   db 13, 10, '$'
msg_greet:     db 'Nice to meet you, $'
msg_bang:      db '!', 13, 10, '$'
msg_press_key: db 'Press any key to exit...', 13, 10, '$'

input_buffer:  db 32, 0
               times 32 db 0
