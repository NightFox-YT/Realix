; © Realix > Calculator Command
; ø Copyright by @liquifield + @Atimenka
; (23.06.26) v0.07
; ================
; ❗️ Зависимости: kernel16/io/print,
;                 kernel16/shell: commands & parse


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
    
    ; Пропуск пробелов перед вторым числом
    call skip_spaces
    cmp byte [si], 0
    je .error_syntax

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

msg_result:     db 'Result: ', 0
str_minus:      db '-', 0
err_syntax:     db '[!] Usage: calc <num1> <+ - * /> <num2>', 0
err_operator:   db '[!] Unknown operator, use + - * /', 0
err_div_zero:   db '[!] Division by zero!', 0
err_overflow:   db '[!] Result too large (overflow)', 0