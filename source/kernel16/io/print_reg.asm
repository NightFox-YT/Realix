; © Realix > Print number (dec / hex/ bcd)
; (11.07.26) v0.09
; ================
; ❗️ Зависимости: kernel16/io/print.asm

; > Вывод значения ax в десятичном виде на экран (Текстовый режим)
; Параметры:
;  - ax: значение регистра
print_dec16:
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
    mov ah, 0x0E  ; TTY mode (Вывод с прокруткой курсора)
    int 0x10
    loop .print_char

.done:
    pop dx
    pop cx
    pop bx
    ret

; > Вывод значения ax в шестнадцатеричном виде на экран (Текстовый режим)
; Параметры:
;  - ax: значение регистра
print_hex16:
    push ax
    push bx
    push cx

    mov bx, ax  ; Создание копии числа в bx
    mov cx, 4   ; Счётчик для 4 цифр

.next_digit:
    rol bx, 4   ; Старший ниббл → младшие 4 бита

    ; Берём цифру из bx (младшие 4 бита из младшего байта bx) и печатаем
    mov al, bl
    call print_hex_digit
    loop .next_digit

.done:
    pop cx
    pop bx
    pop ax
    ret

; > Вывод одной шестнадцатеричной цифры
; Параметры:
;  - al: значение (учитываются только младшие 4 бита)
print_hex_digit:
    push ax

    and al, 0x0F
    cmp al, 9
    jbe .digit        ; Обработка цифры (0-9)
    add al, 'A' - 10  ; Обработка буквы (A-F)
    jmp .print

.digit:
    ; Перевод цифры в ASCII символ
    add al, '0'

.print:
    call print_char

.done:
    pop ax
    ret

; > Вывод значения al в шестнадцатеричном виде (2 цифры) + пробел
; (Полезно для дампов памяти: печатает байт и разделитель одним вызовом)
; Параметры:
;  - al: значение байта
print_byte:
    push ax
    push bx

    ; Сохраняем байт целиком (al будем портить под каждую цифру)
    mov bl, al

    ; Старший ниббл
    mov al, bl
    shr al, 4
    call print_hex_digit

    ; Младший ниббл
    mov al, bl
    call print_hex_digit

    mov al, ' '
    call print_char

    pop bx
    pop ax
    ret

; > Вывод BCD значения al на экран (Текстовый режим)
; Параметры:
;  - al: значение регистра
print_bcd2:
    push ax
    push bx

    ; Сохранение байта числа al в bl
    mov bl, al

    ; Старший ниббл → младшие 4 бита со "страховкой"
    shr al, 4
    and al, 0x0F

    add al, '0'      ; Цифра → ASCII
    call print_char  ; Печать первой цифры
    
    mov al, bl       ; Берём сохранённый байт заново

    ; Младший ниббл → младшие 4 бита
    and al, 0x0F
    add al, '0'      ; Цифра → ASCII
    call print_char  ; Печать второй цифры

    pop bx
    pop ax
    ret