; © Realix > Parsing
; (11.07.26) v0.09
; ================

; > Обработка десятичного числа (uint16)
; Параметры:
;  - ds:si: строка с числом
; Вывод:
;  - ax: число (uint16)
;  - si: указывает на след. символ за числом
;  - CF: 0 (успех), 1 (ошибка/переполнение)
parse_uint16:
    push bx
    push cx
    push dx

    ; Сброс параметров
    xor ax, ax  ; Накопитель результата
    xor cx, cx  ; Счётчик цифр

.next_digit:
    ; Загрузка текущего символа
    xor dx, dx
    mov dl, [si]

    ; Проверка, что ASCII символ является цифрой
    cmp dl, '0'
    jb .finish
    cmp dl, '9'
    ja .finish

    sub dl, '0'  ; ASCII → Цифра (0-9)
    push dx      ; *Cохраняем цифру

    ; Переход к след. разряду числа
    mov bx, 10
    mul bx

    ; Проверка переполнения
    or dx, dx
    pop dx      ; *Восстанавливаем цифру в dx
    jnz .fail   ; Произведение не влезло в 16 битное число

    add ax, dx  ; ax += цифра
    jc .fail    ; Сумма не влезла в 16-битное число

    ; Переход к след. символу (цифре)
    inc si
    inc cx
    jmp .next_digit

.finish:
    ; Проверка на наличие цифр после парсинга
    test cx, cx
    jz .fail

.done:
    clc
    jmp .return

.fail:
    stc

.return:
    pop dx
    pop cx
    pop bx
    ret