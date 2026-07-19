; © Realix > Print CTRL chars
; (15.07.26) v0.1
; ================

; Основные константы
%include 'shared/config.asm'

; Управляющие символы
%define BEEP_CHAR     0x07
%define SQUARE_CHAR   0xFE

; > Вывод символов "\n\r" на экран (Текстовый режим)
print_new_line:
    push ax
    push bx

    ; Настройка TTY mode, номера страницы и цвета
    mov ah, 0x0E
    xor bx, bx

    ; Вывод символов "\n\r"
    mov al, 0x0D
    int 0x10
    mov al, 0x0A
    int 0x10

    pop bx
    pop ax
    ret

; > Перевод строки, только если курсор не в начале строки
print_new_line_if_needed:
    push ax
    push bx
    push cx
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
    pop cx
    pop bx
    pop ax
    ret

; > Вывод невидимого символа "BEL" на экран (Текстовый режим)
print_beep_char:
    push ax
    push bx

    ; Настройка TTY mode, номера страницы и цвета
    mov ah, 0x0E
    xor bx, bx

    ; Вывод символа "BEL"
    mov al, BEEP_CHAR
    int 0x10

    pop bx
    pop ax
    ret

; > Вывод символа "■" на экран (Текстовый режим)
print_square_char:
    push ax
    push bx

    ; Настройка TTY mode, номера страницы и цвета
    mov ah, 0x0E
    xor bx, bx

    ; Вывод символа "■"
    mov al, SQUARE_CHAR
    int 0x10

    pop bx
    pop ax
    ret