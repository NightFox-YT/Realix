; © Realix > IO: Print
; (27.07.26) v0.1
; ================
; ❗️ В режиме PRINT_MINIMAL (Bootix) доступно только `print`

; Управляющие символы
BEEP_CHAR   equ 0x07
SQUARE_CHAR equ 0xFE


; > Вывод строки на экран (Текстовый режим)
; Параметры:
;  - ds:si: адрес строки
print:
    push si
    push ax
    push bx

    mov ah, 0x0E  ; TTY mode (Вывод с прокруткой курсора)
    xor bx, bx    ; Сброс номера страницы и цвета

.next_char:
    lodsb         ; Загрузка символа (si > al)
    test al, al   ; Достигнут ли конец строки?
    jz .done

    int 0x10
    jmp .next_char

.done:
    pop bx
    pop ax
    pop si
    ret


; > Блок "Расширенный вывод" (Не для Bootix)
%ifndef PRINT_MINIMAL


; > Вывод символа из al на экран (Текстовый режим)
; Параметры:
;  - al: символ
print_char:
    push ax
    push bx

    mov ah, 0x0E  ; TTY mode (Вывод с прокруткой курсора)
    xor bx, bx    ; Сброс номера страницы и цвета
    int 0x10

    pop bx
    pop ax
    ret


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
; ❗️ Зависимости: bios-api/io/cursor
print_new_line_if_needed:
    push dx

    ; Запрос позиции курсора: dh - строка, dl - колонка
    call get_cursor_pos

    ; Курсор уже в начале строки - перевод не нужен
    test dl, dl
    jz .done

    call print_new_line

.done:
    pop dx
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


%endif
