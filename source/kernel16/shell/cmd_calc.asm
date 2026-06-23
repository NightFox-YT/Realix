; © Realix > Calculator Command
; (23.06.26) v0.07
; ================
; ❗️ Зависимости: kernel16/io/print (Модуль), kernel16/shell/commands.asm


; > Команда простого калькулятора
cmd_calc:
    push ax
    push bx
    push cx
    push dx
    push si
    push di

.skip_spaces_after_cmd:
    ; Пропуск пробелов до первого числа
    call skip_spaces
    cmp byte [si], 0
    je .error_syntax

    ; Парсинг первого числа
    call parse_number
    jc .error_syntax
    mov [.num1], ax

    ; Пропуск пробелов перед оператором
    call skip_spaces
    cmp byte [si], 0
    je .error_syntax

    ; Читаем оператор
    lodsb
    mov [.operator], al
    
    ; Пропуск пробелов перед вторым числом
    call skip_spaces
    cmp byte [si], 0
    je .error_syntax

    ; Парсинг второго числа
    call parse_number
    jc .error_syntax
    mov [.num2], ax

    ; Вычисление в зависимости от оператора
    mov ax, [.num1]
    mov bx, [.num2]

    cmp byte [.operator], '+'
    je .do_add
    cmp byte [.operator], '-'
    je .do_sub
    cmp byte [.operator], '*'
    je .do_mul
    cmp byte [.operator], '/'
    je .do_div

    jmp .error_operator

; > Выполнение действий с проверкой переполнения (jo)
.do_add:
    add ax, bx
    jo .error_overflow
    jmp .show_result

.do_sub:
    sub ax, bx
    jo .error_overflow
    jmp .show_result

.do_mul:
    mul bx
    jo .error_overflow
    jmp .show_result

.do_div:
    ; Проверка деления на 0
    cmp bx, 0
    je .error_div_zero

    xor dx, dx
    div bx
    jmp .show_result

.show_result:
    ; "Result: "
    mov si, msg_result
    call print

    ; Выводим результат (сохранён в ax)
    call print_reg
    jmp .done

.error_div_zero:
    mov si, err_div_zero
    call print
    jmp .done

.error_overflow:
    mov si, err_overflow
    call print
    jmp .done

.error_operator:
    mov si, err_operator
    call print
    jmp .done

.error_syntax:
    mov si, err_syntax
    call print

.done:
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Локальные переменные
.num1:      dw 0
.num2:      dw 0
.operator:  db 0


; > Парсинг десятичного числа
; Параметры:
;  - si: строка
; Выход:
;  - ax: число
;  - CF: 0 (успех) / 1 (ошибка)
parse_number:
    push bx
    push cx
    push dx

    ; Сброс параметров
    xor ax, ax
    mov cx, 10

.loop:
    ; Загрузка цифры
    xor dx, dx
    mov dl, [si]

    ; Проверка, что ASCII символ является цифрой
    cmp dl, '0'
    jb .check_done
    cmp dl, '9'
    ja .check_done

    sub dl, '0'      ; dl теперь цифра
    push dx          ; *Cохраняем dx (dl - цифра, dh - 0)
    mul cx           ; AX *= 10 (Переходим к след. разряду числа)
    pop dx           ; *Восстанавливаем dx (dl - цифра, dh - 0)
    add ax, dx       ; AX += цифра
    jo .error

    ; Переход к след. символу (цифре)
    inc si
    jmp .loop

.check_done:
    test ax, ax
    jz .error

    clc
    jmp .exit

.error:
    stc

.exit:
    pop dx
    pop cx
    pop bx
    ret

msg_result:     db 'Result: ', 0
err_syntax:     db '[!] Usage: calc <num1> <+ - * /> <num2>', 0
err_operator:   db '[!] Unknown operator, use + - * /', 0
err_div_zero:   db '[!] Division by zero!', 0
err_overflow:   db '[!] Result too large (overflow)', 0