; © Realix > Commands: Text
; (27.07.26) v0.1
; ================
; ❗️ Зависимости: kernel16-nightly/shell/commands.asm, kernel16-nightly/io: print & print_reg

; Предел кода символа для команды `ascii`
ASCII_MAX_CODE equ 255

; Предел числа повторов для команды `repeat`
REPEAT_MAX_COUNT equ 20

; > Команда печати аргумента задом наперед
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_reverse:
    push ax
    push bx
    push di
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces
    mov bx, si        ; bx - указатель начала строки
    mov di, si        ; di - указатель для поиска конца

.find_end:
    ; Идём до нуль-терминатора
    cmp byte [di], 0
    je .check_empty

    inc di
    jmp .find_end

.check_empty:
    ; Проверка на пустой аргумент (Конец совпал с началом)
    cmp di, bx
    je .done

    ; Устанавливаем di на последний символ строки
    dec di

.print_loop:
    ; Печать символов от конца к началу
    cmp di, bx
    jb .done

    mov al, [di]
    call print_char

    ; Дошли до начала строки
    cmp di, bx
    je .done

    ; Переход к след. символу
    dec di
    jmp .print_loop

.done:
    pop si
    pop di
    pop bx
    pop ax
    ret

; > Команда подсчёта длины аргумента
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_len:
    push ax
    push cx
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces
    xor cx, cx        ; Счётчик символов

.loop:
    ; Идём до нуль-терминатора
    cmp byte [si], 0
    je .print

    ; Переход к след. символу, увеличивая счётчик
    inc si
    inc cx
    jmp .loop

.print:
    ; "Length: " + число
    mov si, msg_len
    call print

    ; Печать длины строки
    mov ax, cx
    call print_dec16

.done:
    pop si
    pop cx
    pop ax
    ret

; > Команда печати аргумента в верхнем регистре
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_upper:
    push ax
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces

.loop:
    lodsb        ; Загрузка символа (si > al)
    test al, al  ; Конец строки?
    jz .done

    call char_to_upper
    call print_char
    jmp .loop

.done:
    pop si
    pop ax
    ret

; > Команда печати аргумента в нижнем регистре
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_lower:
    push ax
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces

.loop:
    lodsb
    test al, al
    jz .done

    call char_to_lower
    call print_char
    jmp .loop

.done:
    pop si
    pop ax
    ret

; > Команда вывода символа по десятичному коду
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_ascii:
    push ax
    push si

    ; Парсинг единственного числового аргумента
    call parse_uint16_arg
    jc .usage

    ; Код должен влезать в один байт
    cmp ax, ASCII_MAX_CODE
    ja .usage

    ; "Char: " + сам символ (al - символ)
    mov si, msg_ascii
    call print
    call print_char
    jmp .done

.usage:
    mov si, msg_ascii_usage
    call print

.done:
    pop si
    pop ax
    ret

; > Команда повтора текста `N` раз
; ❗️ Предел N - REPEAT_MAX_COUNT, иначе ошибка синтаксиса
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_repeat:
    push ax
    push cx
    push si

    ; Проверка существования счётчика повторов
    call require_arg
    jc .usage

    ; Парсинг числа повторов
    call parse_uint16
    jc .usage

    ; Ноль повторов - тихо выходим
    test ax, ax
    jz .done

    ; Верхнее ограничение (cx - счётчик)
    cmp ax, REPEAT_MAX_COUNT
    ja .too_many
    mov cx, ax

    ; Проверка существования текста для повтора
    call require_arg
    jc .usage

.print_loop:
    ; Вывод одного и того же текста `N` раз
    call print
    loop .print_loop
    jmp .done

.usage:
    mov si, msg_repeat_usage
    call print
    jmp .done

.too_many:
    mov si, err_repeat_count
    call print

.done:
    pop si
    pop cx
    pop ax
    ret

; Строки
msg_len:          db 'Length: ', 0
msg_ascii:        db 'Char: ', 0
msg_ascii_usage:  db '[?] Usage: ascii <0-255>', 0
msg_repeat_usage: db '[?] Usage: repeat <1-20> <text>', 0

; Сообщения об ошибках
err_repeat_count: db '[!] Repeat count must be 1..20.', 0