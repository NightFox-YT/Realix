; © Realix > Shell Commands
; ø Вдохновлено @nyxmalware
; (13.06.26) v0.06
; ================
; ❗️ Зависимости: bios-api/memory (модуль), bios-api/network
; TODO:
;  - Возвращение carry_flag при ошибке
;  - В shutdown полагаться не только на APM

; Таблица команд (С названиями)
align 2
cmd_table:
    dw .str_help,     cmd_help
    dw .str_cls,      cmd_cls
    dw .str_clear,    cmd_cls
    dw .str_reboot,   cmd_reboot
    dw .str_shutdown, cmd_shutdown
    dw .str_meminfo,  cmd_meminfo
    dw .str_echo,     cmd_echo
    dw .str_calc,     cmd_calc
    dw .str_beep,     print_beep_char
    dw .str_about,    cmd_about
    dw .str_reverse,  cmd_reverse
    dw .str_len,      cmd_len
    dw .str_upper,    cmd_upper
    dw .str_lower,    cmd_lower
    dw .str_hex,      cmd_hex
    dw .str_ascii,    cmd_ascii
    dw .str_repeat,   cmd_repeat
    dw .str_fib,      cmd_fib
    dw 0, 0

.str_help:     db 'help', 0
.str_cls:      db 'cls', 0
.str_clear:    db 'clear', 0
.str_reboot:   db 'reboot', 0
.str_shutdown: db 'shutdown', 0
.str_meminfo:  db 'meminfo', 0
.str_echo:     db 'echo', 0
.str_calc:     db 'calc', 0
.str_beep:     db 'beep', 0
.str_about:    db 'about', 0
.str_reverse:  db 'reverse', 0
.str_len:      db 'len', 0
.str_upper:    db 'upper', 0
.str_lower:    db 'lower', 0
.str_hex:      db 'hex', 0
.str_ascii:    db 'ascii', 0
.str_repeat:   db 'repeat', 0
.str_fib:      db 'fib', 0


; > Исполнитель команд
; Параметры:
;  - si: указатель на введенную строку
execute_cmd:
    push si
    push di
    push ax
    push bx

.skip_spaces:
    lodsb
    cmp al, ' '
    je .skip_spaces

    ; SI указывает на первый символ команды
    dec si

    ; Проверка на пустую строку после пробелов
    cmp byte [si], 0
    je .done

    ; Инициализируем проход по таблице команд
    mov di, cmd_table

.search_next:
    mov bx, [di]   ; Получаем имя команды по адресу
    test bx, bx    ; Проверяем имя команды на конец таблицы (0)
    jz .not_found
    
    ; Сохраняем указатель на начало ввода пользователя
    push si

.compare_loop:
    ; Загрузка символа из ввода и таблицы
    mov al, [si]
    mov ah, [bx]

    ; Имя команды в таблице закончилась (\0)?
    test ah, ah
    jz .check_match

    ; Сравнение символов ввода и таблицы
    cmp al, ah
    jne .mismatch

    ; Переход к след. символу
    inc si
    inc bx
    jmp .compare_loop

.check_match:
    ; Проверка, что во вводе дальше
    cmp al, 0
    je .match
    cmp al, ' '
    je .match

.mismatch:
    pop si            ; *Восстанавливаем для проверки след. команды
    add di, 4         ; Сдвигаем на следующую запись
    jmp .search_next

.match:
    pop ax            ; *Восстанавливаем push si из стека
    mov ax, [di + 2]  ; Берем адрес функции
    call ax           ; Вызываем функцию команды
    jmp .done

.not_found:
    mov si, err_unknown_cmd
    call print

.done:
    pop bx
    pop ax
    pop di
    pop si
    ret


; > Команда помощи
cmd_help:
    push si

    mov si, msg_help
    call print

    pop si
    ret

; > Команда очистки экрана
%include "kernel16/shell/cmd_cls.asm"

; > Команда перезагрузки
cmd_reboot:
    cli

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0x0000

; > Команда вывода информации о памяти
cmd_meminfo:
    push si
    push di

    ; Показ строк с информацией о памяти
    mov di, PCINFO_ADDR
    call show_lower_memory
    call print_new_line

    mov di, PCINFO_ADDR
    call show_free_memory
    call print_new_line

    mov di, PCINFO_ADDR
    call show_map_entries_cnt

    call print_new_line
    call print_new_line

    ; Небольшая заметка
    mov si, note_meminfo
    call print

    pop di
    pop si
    ret

cmd_shutdown:
    push ax
    push bx
    push cx
    push si

    ; Подключение к APM:
    ; > ax - APM функция с подключением реального режима
    ; > bx - устройство: "System BIOS"
    mov ax, 0x5301
    xor bx, bx
    int 0x15
    jc .error

    ; Установка версии APM
    ; > al - выбор версии
    ; > bx - устройство: "System BIOS"
    ; > cx - запрашиваемая версия APM 1.2
    mov ax, 0x530E
    xor bx, bx
    mov cx, 0x0102
    int 0x15
    jc .error

    ; Команда выключения питания
    ; > al - установка состояния питания
    ; > bx - устройство: "All devices" (все устройства)
    ; > cx - состояние: "Off" (выключить)
    mov ax, 0x5307
    mov bx, 0x0001
    mov cx, 0x0003
    int 0x15
    jc .error
    
    jmp $

.error:
    mov si, err_shutdown
    call print
    
    pop si
    pop cx
    pop bx
    pop ax
    ret

; > Команда "Echo"
; Параметры:
;  - si: указатель на начало аргументов команды
cmd_echo:
    push ax
    push si

    ; Пропускаем пробелы
    call skip_spaces

    ; Если аргументов нет (сразу конец строки)
    cmp byte [si], 0
    jz .done

    ; Вывод сообщения         
    call print

.done:
    pop si
    pop ax

    ret

; > Команда "About"
cmd_about:
    push si

    mov si, msg_about
    call print

    pop si
    ret

; > Команда печати аргумента задом наперед
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
    je .is_empty

    inc di
    jmp .find_end

.is_empty:
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

    pop si
    pop cx
    pop ax
    ret

; > Команда печати аргумента в верхнем регистре
cmd_upper:
    push ax
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces

.loop:
    lodsb        ; Загрузка символа (si > al)
    test al, al  ; Конец строки?
    jz .done

    ; Переводим только a-z, остальное - как есть
    cmp al, 'a'
    jb .emit
    cmp al, 'z'
    ja .emit

    ; Перевод в верхний регистр по сдвигу в таблице ASCII
    sub al, 32

.emit:
    call print_char
    jmp .loop

.done:
    pop si
    pop ax
    ret

; > Команда печати аргумента в нижнем регистре
cmd_lower:
    push ax
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces

.loop:
    lodsb
    test al, al
    jz .done

    ; Переводим только A-Z, остальное - как есть
    cmp al, 'A'
    jb .emit
    cmp al, 'Z'
    ja .emit

    ; Перевод в нижний регистр по сдвигу в таблице ASCII
    add al, 32

.emit:
    call print_char
    jmp .loop

.done:
    pop si
    pop ax
    ret

; > Команда вывода числа в шестнадцатеричном виде
cmd_hex:
    push ax
    push bx
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces
    cmp byte [si], 0
    je .usage

    ; Парсинг числа
    call parse_uint16
    jc .usage
    mov bx, ax         ; Сохраняем число (parse затрёт ax)

    ; После числа не должно быть мусора
    call skip_spaces
    cmp byte [si], 0
    jne .usage

    ; "Hex: 0x" + число
    mov si, msg_hex
    call print

    mov ax, bx
    call print_hex16
    jmp .done

.usage:
    mov si, msg_hex_usage
    call print

.done:
    pop si
    pop bx
    pop ax
    ret

; > Команда вывода символа по десятичному коду
cmd_ascii:
    push ax
    push bx
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces
    cmp byte [si], 0
    je .usage

    ; Парсинг кода символа
    call parse_uint16
    jc .usage

    ; Код должен влезать в один байт
    cmp ax, 255
    ja .usage
    mov bx, ax

    ; После числа не должно быть мусора
    call skip_spaces
    cmp byte [si], 0
    jne .usage

    ; "Char: " + сам символ
    mov si, msg_ascii
    call print
    mov al, bl
    call print_char
    jmp .done

.usage:
    mov si, msg_ascii_usage
    call print

.done:
    pop si
    pop bx
    pop ax
    ret

; > Команда повтора текста `N` раз
; ❗️ Предел N: 20, иначе ошибка синтаксиса
cmd_repeat:
    push ax
    push cx
    push si

    ; Пропуск пробелов до счётчика
    call skip_spaces
    cmp byte [si], 0
    je .usage

    ; Парсинг числа повторов
    call parse_uint16
    jc .usage

    ; Ноль повторов - тихо выходим
    test ax, ax
    jz .done

    ; Верхнее ограничение
    cmp ax, 20
    ja .too_many
    mov cx, ax        ; cx - счётчик

    ; Пропуск пробелов до текста
    call skip_spaces
    cmp byte [si], 0
    je .usage

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

; > Команда вычисления числа Фибоначчи (0-24)
; ❗️ Предел 24, чтобы результат влез в 16 бит (fib(24) = 46368)
cmd_fib:
    push ax
    push bx
    push cx
    push dx
    push si

    ; Пропуск пробелов до аргумента
    call skip_spaces
    cmp byte [si], 0
    je .usage

    ; Парсинг номера
    call parse_uint16
    jc .usage
    push ax

    ; После числа не должно быть мусора
    call skip_spaces
    cmp byte [si], 0
    jne .usage

    ; Ограничение сверху (иначе переполнение uint16)
    pop ax
    cmp ax, 24
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

; > Команда простого калькулятора
%include "kernel16/shell/cmd_calc.asm"

; > Пропуск пробелов до первого символа
; Вывод:
;  - si: указатель на первый символ строки
skip_spaces:
    ; Пропуск пробела c переходом к след. символу
    lodsb
    cmp al, ' '
    je skip_spaces

.done:
    ; Возвращаем si назад на первый символ строки
    dec si
    ret

err_unknown_cmd:  db '[!] Unknown command, write help for list of commands.', 0
err_shutdown:     db '[!] PC shutdown failed! (No APM)', 0
err_repeat_count: db '[!] Repeat count must be 1..20.', 0
err_fib_range:    db '[!] Fib argument must be 0..24.', 0

; Сообщения
msg_help:
    db 'Commands:', ENTER
    db '  [Base]', ENTER
    db '> help      - Show this manual', ENTER
    db '> clear/cls - Clear screen', ENTER
    db '> echo <t>  - Print <text> to console', ENTER
    db '> about     - Show system info', ENTER
    db '> beep      - Beep via BIOS speaker', ENTER
    db '> meminfo   - Display RAM configuration', ENTER
    db '  [Text]', ENTER
    db '> len <t>     - Length of <text>', ENTER
    db '> upper <t>   - <text> to upper case', ENTER
    db '> lower <t>   - <text> to lower case', ENTER
    db '> reverse <t> - Reverse <text>', ENTER
    db '  [Numbers]', ENTER
    db '> calc <num1> <+ - * /> <num2> - Simple calculator (positive only)', ENTER
    db '> hex <num>         - Show <num> in hexadecimal', ENTER
    db '> ascii <0-255>     - Print char by ASCII code', ENTER
    db '> repeat <1-20> <t> - Repeat <text> N times', ENTER
    db '> fib <0-24>        - Nth Fibonacci number', ENTER
    db '  [Power]', ENTER
    db '> reboot   - Reboot PC', ENTER
    db '> shutdown - Power off PC', 0
msg_about:
    db '> Realix version: ', OS_VERSION, ENTER
    db '> Realix is a lightweight hybrid x86 OS.', ENTER
    db 'It supports a built-in boot switcher that lets users choose:', ENTER
    db '1. 16-bit Real Mode kernel for legacy compatibility', ENTER
    db '2. 32-bit Protected Mode kernel for high performance.', 0

msg_len:          db 'Length: ', 0
msg_hex:          db 'Hex: 0x', 0
msg_hex_usage:    db 'Usage: hex <num>', 0
msg_ascii:        db 'Char: ', 0
msg_ascii_usage:  db 'Usage: ascii <0-255>', 0
msg_repeat_usage: db 'Usage: repeat <1-20> <text>', 0
msg_fib:          db 'Fib: ', 0
msg_fib_usage:    db 'Usage: fib <0-24>', 0

note_meminfo: db 'Note: In Real mode you can access only up to 1 MB RAM.', 0

; Буфер ввода
input_str: times 64 db 0
