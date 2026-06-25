; © Realix > Print register AX (dec)
; (28.03.26) v0.04
; ================

; > Вывод значения ax в десятичном виде на экран (Текстовый режим)
; Параметры:
;  - ax: значение регистра
print_reg:
    push bx
    push cx
    push dx

    mov bx, 10    ; Делитель (Для перевода в десятичный вид)
    xor cx, cx    ; Счётчик цифр

.next_digit:
    xor dx, dx    ; Обнуление dx с ASCII символом
    div bx        ; ax - частное (dx - остаток: цифра)
    add dl, 0x30  ; Цифра → ASCII
    push dx       ; Сохраняем цифру в стеке

    ; Переход к след. цифре
    inc cx
    test ax, ax
    jnz .next_digit

.print_char:
    pop ax        ; Достаём цифру из стека
%ifdef KERNEL_CONSOLE
    call console_putc
%else
    mov ah, 0x0E  ; TTY mode (Вывод с прокруткой курсора)
    int 0x10
%endif
    loop .print_char

.done:
    pop dx
    pop cx
    pop bx
    ret