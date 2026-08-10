; © Realix > Command: Key
; (29.07.26) v0.11
; ================
; ❗️ Зависимости: kernel16/io/print_reg & print

; > Команда ожидания клавиши и вывода ASCII/scancode
cmd_key:
    push ax
    push bx
    push si

    ; "Нажмите любую клавишу..."
    mov si, msg_key_wait
    call print

    ; Функция BIOS: Ожидание нажатия
    xor ah, ah
    int 0x16

    ; Сохранение в промежуточный регистр
    mov bx, ax

    ; "ASCII: " и выводим нижний байт
    mov si, msg_key_ascii
    call print
    mov ax, bx
    xor ah, ah
    call print_dec16

    ; "Scancode: " и выводим верхний байт
    mov si, msg_key_scan
    call print
    mov al, ah
    call print_byte

    pop si
    pop bx
    pop ax
    ret

; Строки
msg_key_wait:  db 'Press any key...', ENTER, 0
msg_key_ascii: db 'ASCII: ', 0
msg_key_scan:  db ' | Scancode: ', 0