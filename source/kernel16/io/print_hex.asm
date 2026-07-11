; © Realix > Print HEX (16-bit)
; Исправленная версия
; ===================

%ifndef PRINT_HEX_ASM
%define PRINT_HEX_ASM


; > Вывод 16-битного числа в HEX
; Вход:
;  - ax: число
print_hex:
    push ax
    push cx
    push dx

    mov dx, ax
    mov cx, 4

.loop:
    rol dx, 4

    mov al, dl
    and al, 0x0F
    call print_hex_digit

    loop .loop

.done:
    pop dx
    pop cx
    pop ax
    ret


; > Вывод одного байта в HEX
; Вход:
;  - al: байт
print_byte:
    push ax
    push dx

    ; Сохраняем исходное значение.
    mov dl, al

    ; Старшая тетрада.
    mov al, dl
    shr al, 4
    and al, 0x0F
    call print_hex_digit

    ; Младшая тетрада.
    mov al, dl
    and al, 0x0F
    call print_hex_digit

    mov al, ' '
    call print_char

    pop dx
    pop ax
    ret


; > Вывод одной шестнадцатеричной цифры
; Вход:
;  - al: значение 0..15
print_hex_digit:
    and al, 0x0F
    add al, '0'

    cmp al, '9'
    jbe .print

    add al, 'A' - '9' - 1

.print:
    call print_char
    ret

%endif
