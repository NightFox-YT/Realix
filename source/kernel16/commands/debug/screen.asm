; © Realix > Command: Screen
; (29.07.26) v0.11
; ================
; ❗️ Зависимости: kernel16/io/print_reg & print

; > Команда показа информаии о видеорежиме,
; кол-во столбцов и текущем номере страницы
cmd_screen:
    push ax
    push bx
    push cx
    push si

    ; Функция BIOS
    mov ah, 0x0F
    int 0x10

    ; Сохраняем в промежуточный регистр
    mov cx, ax

    ; Вывод номера видеорежима (нижний байт)
    mov si, msg_screen_mode
    call print
    movzx ax, cl
    call print_hex16

    ; Вывод кол-ва столбцов (верхний байт)
    mov si, msg_screen_cols
    call print
    movzx ax, ch
    call print_dec16

    ; Вывод номера текущей страницы (верхий байт bx)
    mov si, msg_screen_page
    call print
    movzx ax, bh
    call print_dec16

    pop si
    pop cx
    pop bx
    pop ax
    ret

; Строки
msg_screen_mode:  db 'Video mode: ', 0
msg_screen_cols:  db ' | Columns: ', 0
msg_screen_page:  db ' | Page: ', 0