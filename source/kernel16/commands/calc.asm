; © Realix > Command: Calculator
; ø Copyright by @liquifield + @Atimenka
; (27.07.26) v0.1
; ================
; ❗️ Зависимости: kernel16/io: print & print_reg,
;                 kernel16/shell: commands & parse

; > Команда простого калькулятора: calc <a> <+ - * /> <b>
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_calc:
    push ax
    push bx
    push dx
    push si

    ; Проверка существования первого числа
    call require_arg
    jc .error_syntax

    ; Парсинг первого числа
    call parse_uint16
    jc .error_syntax
    mov [.num1], ax

    ; Пропуск пробелов перед оператором
    call skip_spaces
    cmp byte [si], 0
    je .error_syntax

    ; Читаем оператор
    lodsb
    mov [.operator], al

    ; Алиасы операторов без Shift: , -> + . -> - ' -> *
    cmp al, ','
    jne .not_comma
    mov byte [.operator], '+'
.not_comma:
    cmp al, '.'
    jne .not_dot
    mov byte [.operator], '-'
.not_dot:
    cmp al, 0x27  ; '
    jne .not_quote
    mov byte [.operator], '*'
.not_quote:

    ; Проверка существования второго числа
    call require_arg
    jc .error_syntax

    ; Парсинг второго числа
    call parse_uint16
    jc .error_syntax
    mov [.num2], ax

    ; После второго числа не должно быть мусора
    call skip_spaces
    cmp byte [si], 0
    jne .error_syntax

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

; > Выполнение действий с проверкой безнакового переполнения
.do_add:
    add ax, bx
    jc .error_overflow
    jmp .show_result

.do_sub:
    ; Проверяем какое число получится в результате
    cmp ax, bx
    jb .sub_negative
    sub ax, bx
    jmp .show_result

.sub_negative:
    ; Печатаем "Result: -" и модуль разности чисел
    mov si, msg_result
    call print
    mov si, str_minus
    call print

    ; Вычитание по модулю со сменой чисел в регистрах
    xchg ax, bx
    sub ax, bx
    call print_dec16
    jmp .done

.do_mul:
    mul bx
    jc .error_overflow
    jmp .show_result

.do_div:
    ; Проверка деления на 0
    test bx, bx
    je .error_div_zero

    xor dx, dx
    div bx
    jmp .show_result

.show_result:
    ; "Result: "
    mov si, msg_result
    call print

    ; Выводим результат (сохранён в ax)
    call print_dec16
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
    mov si, msg_calc_usage
    call print

.done:
    pop si
    pop dx
    pop bx
    pop ax
    ret

; Локальные переменные
.num1:     dw 0
.num2:     dw 0
.operator: db 0

; Строки
msg_result:     db 'Result: ', 0
str_minus:      db '-', 0
msg_calc_usage: db "[?] Usage: calc <num1> <+ - * /> <num2> (or , . ' without Shift)", 0
err_operator:   db "[!] Unknown operator, use + - * / (or , . ' without Shift)", 0
err_div_zero:   db '[!] Division by zero!', 0
err_overflow:   db '[!] Result too large (overflow)', 0