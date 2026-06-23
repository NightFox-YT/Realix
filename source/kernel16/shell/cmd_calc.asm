; © Realix > Calculator Command
; (23.06.26) v0.06+
; ================
; ❗️ Зависимости: kernel16/io/print.asm, kernel16/io/print_nl.asm, kernel16/io/print_reg.asm
; Формат: calc <num1> <op> <num2>
; Примеры: calc 12+5, calc 12 + 5, calc 12 +5

cmd_calc:
    push ax
    push bx
    push cx
    push dx
    push si
    push di

    ; --- Пропускаем имя команды "calc" ---
.skip_cmd_name:
    lodsb
    cmp al, ' '
    je .skip_spaces_after_cmd
    cmp al, 0
    je .error_syntax
    jmp .skip_cmd_name

.skip_spaces_after_cmd:
    call skip_spaces_calc
    cmp byte [si], 0
    je .error_syntax

    ; --- Парсим первое число ---
    call parse_number
    jc .error_syntax
    mov [num1], ax

    ; --- Пропускаем пробелы перед оператором (если есть) ---
    call skip_spaces_calc
    cmp byte [si], 0
    je .error_syntax

    ; --- Читаем оператор ---
    lodsb
    mov [operator], al

    ; Проверка: оператор должен быть + - * /
    cmp byte [operator], '+'
    je .valid_op
    cmp byte [operator], '-'
    je .valid_op
    cmp byte [operator], '*'
    je .valid_op
    cmp byte [operator], '/'
    je .valid_op
    jmp .error_operator

.valid_op:
    ; --- Пропускаем пробелы перед вторым числом (если есть) ---
    call skip_spaces_calc
    cmp byte [si], 0
    je .error_syntax

    ; --- Парсим второе число ---
    call parse_number
    jc .error_syntax
    mov [num2], ax

    ; --- Вычисляем ---
    mov ax, [num1]
    mov bx, [num2]

    cmp byte [operator], '+'
    je .do_add
    cmp byte [operator], '-'
    je .do_sub
    cmp byte [operator], '*'
    je .do_mul
    cmp byte [operator], '/'
    je .do_div

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
    cmp dx, 0
    jne .error_overflow
    jmp .show_result

.do_div:
    cmp bx, 0
    je .error_div_zero
    xor dx, dx
    div bx
    jmp .show_result

.show_result:
    push ax
    mov si, msg_result
    call print
    pop ax
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


; --- Вспомогательная: пропуск пробелов ---
skip_spaces_calc:
    cmp byte [si], ' '
    jne .done_skip
    inc si
    jmp skip_spaces_calc
.done_skip:
    ret


; --- Вспомогательная: парсинг десятичного числа ---
; Вход:  si — строка
; Выход: ax — число, CF=0 (успех) / CF=1 (ошибка)

parse_number:
    push bx
    push cx
    push dx

    xor ax, ax
    xor bx, bx
    mov cx, 10

.parse_loop:
    xor dx, dx
    mov dl, [si]
    cmp dl, '0'
    jb .check_done
    cmp dl, '9'
    ja .check_done

    sub dl, '0'        ; dl = цифра
    push dx            ; сохраняем ЧИСТЫЙ dx (цифра, dh=0)
    mul cx             ; AX *= 10
    pop dx             ; dx = цифра (dh=0)
    add ax, dx         ; AX += цифра
    jo .parse_error
    inc si
    mov bx, 1
    jmp .parse_loop

.check_done:
    test bx, bx
    jz .parse_error
    clc
    jmp .parse_exit

.parse_error:
    stc

.parse_exit:
    pop dx
    pop cx
    pop bx
    ret

; --- Данные ---
num1:      dw 0
num2:      dw 0
operator:  db 0

msg_result:     db 'Result: ', 0
err_syntax:     db '[!] Usage: calc <num1> <+|-|*|/> <num2>', 0
err_operator:   db '[!] Unknown operator, use + - * /', 0
err_div_zero:   db '[!] Division by zero!', 0
err_overflow:   db '[!] Result too large (overflow)', 0
