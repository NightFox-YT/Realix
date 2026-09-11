; © Realix > IO: Print CTRL chars
; (27.07.26) v0.1
; ================
; ❗️ Зависимости: kernel16-nightly/io/print

; Управляющие символы
BEEP_CHAR   equ 0x07
SQUARE_CHAR equ 0xFE

; > Вывод символов "\r\n" на экран (Текстовый режим)
print_new_line:
    push ax
    push bx

    ; Настройка TTY mode, номера страницы и цвета
    mov ah, 0x0E
    xor bx, bx

    ; Вывод символов "\r\n"
    mov al, 0x0D
    int 0x10
    mov al, 0x0A
    int 0x10

    pop bx
    pop ax
    ret

; > Перевод строки, если курсор не в начале строки (Текстовый режим)
print_new_line_if_needed:
    push ax
    push bx
    push dx

    ; Запрос позиции курсора: dh - строка, dl - колонка
    mov ah, 0x03
    xor bh, bh
    int 0x10

    ; Курсор уже в начале строки - перевод не нужен
    test dl, dl
    jz .done

    call print_new_line

.done:
    pop dx
    pop bx
    pop ax
    ret

; > Вывод невидимого символа "BEL" на экран (Текстовый режим)
print_beep_char:
    push ax

    ; Вывод символа "BEL"
    mov al, BEEP_CHAR
    call print_char

    pop ax
    ret

; > Вывод символа "■" на экран (Текстовый режим)
print_square_char:
    push ax

    ; Вывод символа "■"
    mov al, SQUARE_CHAR
    call print_char

    pop ax
    ret
