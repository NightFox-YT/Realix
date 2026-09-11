; © Realix > Commands: Numbers
; (27.07.26) v0.1
; ================
; ❗️ Зависимости: kernel16-nightly/shell/commands.asm, kernel16-nightly/io: print & print_reg

; Предел индекса Фибоначчи (Результат должен влезть в uint16)
; > fib(24) = 46368, fib(25) = 75025 (Переполнение)
FIB_MAX_INDEX equ 24

; > Команда вычисления числа Фибоначчи (0-24)
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_fib:
    push ax
    push bx
    push cx
    push dx
    push si

    ; Парсинг единственного числового аргумента
    call parse_uint16_arg
    jc .usage

    ; Ограничение сверху (иначе переполнение uint16)
    cmp ax, FIB_MAX_INDEX
    ja .too_big

    ; Начальная пара: ax = fib(n-1), bx = fib(n) = 1
    mov cx, ax
    xor ax, ax
    mov bx, 1

    ; Позиция 0 в последовательности Фибоначчи известна заранее
    test cx, cx
    jz .print

.loop:
    ; Линейный счёт числа в последовательности (dx = ax + bx)
    mov dx, ax
    add dx, bx

    ; Обновляем позиции: fib(n-1), fib(n)
    mov ax, bx
    mov bx, dx
    loop .loop

.print:
    ; "Fib: " + результат (уже в ax)
    mov si, msg_fib
    call print
    call print_dec16
    jmp .done

.usage:
    mov si, msg_fib_usage
    call print
    jmp .done

.too_big:
    mov si, err_fib_range
    call print

.done:
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; > Команда вывода числа в шестнадцатеричном виде
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_hex:
    push ax
    push si

    ; Парсинг единственного числового аргумента
    call parse_uint16_arg
    jc .usage

    ; "Hex: 0x" + число (ax - число)
    mov si, msg_hex
    call print
    call print_hex16
    jmp .done

.usage:
    mov si, msg_hex_usage
    call print

.done:
    pop si
    pop ax
    ret

; Строки
msg_fib:       db 'Fib: ', 0
msg_fib_usage: db '[?] Usage: fib <0-24>', 0
msg_hex:       db 'Hex: 0x', 0
msg_hex_usage: db '[?] Usage: hex <num>', 0

; Сообщения об ошибках
err_fib_range: db '[!] Fib argument must be 0..24.', 0