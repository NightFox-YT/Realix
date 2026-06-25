; Realix Kernel16 shell commands
; Small BIOS-based commands for the 16-bit shell.
; This file is included from kernel16/kernel.asm.

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
    dw .str_ver,      cmd_ver
    dw .str_about,    cmd_about
    dw .str_beep,     cmd_beep
    dw .str_time,     cmd_time
    dw .str_date,     cmd_date
    dw .str_calc,     cmd_calc
    dw .str_key,      cmd_key
    dw .str_ticks,    cmd_ticks
    dw .str_rand,     cmd_rand
    dw .str_len,      cmd_len
    dw .str_upper,    cmd_upper
    dw .str_lower,    cmd_lower
    dw .str_reverse,  cmd_reverse
    dw .str_wait,     cmd_wait
    dw .str_cursor,   cmd_cursor
    dw .str_screen,   cmd_screen
    dw .str_uptime,   cmd_uptime
    dw .str_hex,      cmd_hex
    dw .str_ascii,    cmd_ascii
    dw .str_repeat,   cmd_repeat
    dw .str_history,  cmd_history
    dw .str_boot,     cmd_boot
    dw .str_segs,     cmd_segs
    dw .str_mode3,    cmd_mode3
    dw .str_mode50,   cmd_mode50
    dw .str_banner,   cmd_banner
    dw .str_fib,      cmd_fib
    dw .str_panic,    cmd_panic
    dw .str_divzero,  cmd_divzero
    dw .str_halt,     cmd_halt
    dw 0, 0

.str_help:     db 'help', 0
.str_cls:      db 'cls', 0
.str_clear:    db 'clear', 0
.str_reboot:   db 'reboot', 0
.str_shutdown: db 'shutdown', 0
.str_meminfo:  db 'meminfo', 0
.str_echo:     db 'echo', 0
.str_ver:      db 'ver', 0
.str_about:    db 'about', 0
.str_beep:     db 'beep', 0
.str_time:     db 'time', 0
.str_date:     db 'date', 0
.str_calc:     db 'calc', 0
.str_key:      db 'key', 0
.str_ticks:    db 'ticks', 0
.str_rand:     db 'rand', 0
.str_len:      db 'len', 0
.str_upper:    db 'upper', 0
.str_lower:    db 'lower', 0
.str_reverse:  db 'reverse', 0
.str_wait:     db 'wait', 0
.str_cursor:   db 'cursor', 0
.str_screen:   db 'screen', 0
.str_uptime:   db 'uptime', 0
.str_hex:      db 'hex', 0
.str_ascii:    db 'ascii', 0
.str_repeat:   db 'repeat', 0
.str_history:  db 'history', 0
.str_boot:     db 'boot', 0
.str_segs:     db 'segs', 0
.str_mode3:    db 'mode3', 0
.str_mode50:   db 'mode50', 0
.str_banner:   db 'banner', 0
.str_fib:      db 'fib', 0
.str_panic:    db 'panic', 0
.str_divzero:  db 'divzero', 0
.str_halt:     db 'halt', 0


; Execute command from DS:SI
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
    ; Compare command names. Only the command part is case-insensitive.
    mov al, [si]
    mov ah, [bx]

    ; End of command name in table?
    test ah, ah
    jz .check_match

    ; A-Z -> a-z for command matching.
    cmp al, 'A'
    jb .compare_char
    cmp al, 'Z'
    ja .compare_char
    add al, 32

.compare_char:
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
    ; Keep shell running after every normal command.
    clc
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

    pop si
    pop di
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

.skip_spaces:
    lodsb
    cmp al, ' '
    je .skip_spaces

    ; Если аргументов нет (сразу конец строки)
    test al, al
    jz .done

    ; Возвращаем si назад на первый символ аргументов, выводим
    dec si                 
    call print

.done:
    pop si
    pop ax

    ret

; > Команда версии
cmd_ver:
    push si

    mov si, msg_version
    call print

    pop si
    ret

; > Краткая информация о системе
cmd_about:
    push si

    mov si, msg_about
    call print

    pop si
    ret

; > Команда звукового сигнала BIOS TTY BEL
cmd_beep:
    push ax
    push bx

    mov ah, 0x0E
    xor bx, bx
    mov al, BEEP_CHAR
    int 0x10

    pop bx
    pop ax
    ret

; > Команда вывода времени RTC/BIOS
cmd_time:
    push ax
    push bx
    push cx
    push dx
    push si

    mov ah, 0x02
    int 0x1A
    jc .error

    mov si, msg_time
    call print

    mov al, ch        ; часы, BCD
    call print_bcd2
    mov al, ':'
    call print_char_al
    mov al, cl        ; минуты, BCD
    call print_bcd2
    mov al, ':'
    call print_char_al
    mov al, dh        ; секунды, BCD
    call print_bcd2
    jmp .done

.error:
    mov si, err_rtc
    call print

.done:
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; > Команда вывода даты RTC/BIOS
cmd_date:
    push ax
    push bx
    push cx
    push dx
    push si

    mov ah, 0x04
    int 0x1A
    jc .error

    mov si, msg_date
    call print

    mov al, dl        ; день, BCD
    call print_bcd2
    mov al, '.'
    call print_char_al
    mov al, dh        ; месяц, BCD
    call print_bcd2
    mov al, '.'
    call print_char_al
    mov al, ch        ; век, BCD
    call print_bcd2
    mov al, cl        ; год, BCD
    call print_bcd2
    jmp .done

.error:
    mov si, err_rtc
    call print

.done:
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; > Простой калькулятор: calc <a> <+|-|*|/> <b>
; Ограничения: ввод и результат — 16-bit unsigned (для вычитания возможен '-' перед результатом)
cmd_calc:
    push ax
    push bx
    push cx
    push dx
    push si

    call skip_spaces
    cmp byte [si], 0
    je .usage

    call parse_uint16
    jc .bad_syntax
    mov [calc_left], ax

    call skip_spaces
    mov al, [si]
    cmp al, '+'
    je .operator_ok
    cmp al, '-'
    je .operator_ok
    cmp al, '*'
    je .operator_ok
    cmp al, '/'
    je .operator_ok
    jmp .bad_syntax

.operator_ok:
    mov [calc_op], al
    inc si

    call skip_spaces
    call parse_uint16
    jc .bad_syntax
    mov bx, ax

    call skip_spaces
    cmp byte [si], 0
    jne .bad_syntax

    mov ax, [calc_left]
    mov dl, [calc_op]

    cmp dl, '+'
    je .add
    cmp dl, '-'
    je .sub
    cmp dl, '*'
    je .mul
    cmp dl, '/'
    je .div
    jmp .bad_syntax

.add:
    add ax, bx
    jc .overflow
    jmp .print_positive

.sub:
    cmp ax, bx
    jae .sub_positive

    ; отрицательный результат: печатаем знак и модуль числа
    mov ax, bx
    sub ax, [calc_left]
    mov si, msg_calc_result
    call print
    mov al, '-'
    call print_char_al
    call print_reg
    jmp .done

.sub_positive:
    sub ax, bx
    jmp .print_positive

.mul:
    mul bx
    or dx, dx
    jnz .overflow
    jmp .print_positive

.div:
    test bx, bx
    jz .div_zero
    xor dx, dx
    div bx
    jmp .print_positive

.print_positive:
    mov si, msg_calc_result
    call print
    call print_reg
    jmp .done

.usage:
    mov si, msg_calc_usage
    call print
    jmp .done

.bad_syntax:
    mov si, err_calc_syntax
    call print
    jmp .done

.overflow:
    mov si, err_calc_overflow
    call print
    jmp .done

.div_zero:
    mov si, err_calc_div_zero
    call print

.done:
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret


; > Ожидание клавиши и вывод ASCII/scan code
cmd_key:
    push ax
    push bx
    push si

    mov si, msg_key_wait
    call print

    xor ah, ah
    int 0x16
    mov bx, ax

    mov si, msg_key_ascii
    call print
    mov ax, bx
    xor ah, ah
    call print_reg

    mov si, msg_key_scan
    call print
    mov ax, bx
    mov al, ah
    xor ah, ah
    call print_reg

    pop si
    pop bx
    pop ax
    ret

; > Количество BIOS timer ticks с полуночи
cmd_ticks:
    push ax
    push bx
    push cx
    push dx
    push si

    xor ah, ah
    int 0x1A

    mov bx, dx

    mov si, msg_ticks_hi
    call print
    mov ax, cx
    call print_reg

    mov si, msg_ticks_low
    call print
    mov ax, bx
    call print_reg

    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; > Простое псевдослучайное число на базе BIOS ticks
cmd_rand:
    push ax
    push cx
    push dx
    push si

    xor ah, ah
    int 0x1A

    mov ax, dx
    xor ax, cx
    add ax, [rand_state]
    rol ax, 1
    add [rand_state], ax

    mov si, msg_rand
    call print
    call print_reg

    pop si
    pop dx
    pop cx
    pop ax
    ret

; > Длина строки: len [text]
cmd_len:
    push ax
    push cx
    push si

    call skip_spaces
    xor cx, cx

.loop:
    cmp byte [si], 0
    je .print
    inc si
    inc cx
    jmp .loop

.print:
    mov si, msg_len
    call print
    mov ax, cx
    call print_reg

    pop si
    pop cx
    pop ax
    ret

; > Печать аргументов в верхнем регистре: upper [text]
cmd_upper:
    push ax
    push si

    call skip_spaces

.loop:
    lodsb
    test al, al
    jz .done

    cmp al, 'a'
    jb .print_char
    cmp al, 'z'
    ja .print_char
    sub al, 32

.print_char:
    call print_char_al
    jmp .loop

.done:
    pop si
    pop ax
    ret

; > Печать аргументов в нижнем регистре: lower [text]
cmd_lower:
    push ax
    push si

    call skip_spaces

.loop:
    lodsb
    test al, al
    jz .done

    cmp al, 'A'
    jb .print_char
    cmp al, 'Z'
    ja .print_char
    add al, 32

.print_char:
    call print_char_al
    jmp .loop

.done:
    pop si
    pop ax
    ret

; > Печать аргументов задом наперед: reverse [text]
cmd_reverse:
    push ax
    push bx
    push di
    push si

    call skip_spaces
    mov bx, si
    mov di, si

.find_end:
    cmp byte [di], 0
    je .print_loop_prepare
    inc di
    jmp .find_end

.print_loop_prepare:
    cmp di, bx
    je .done
    dec di

.print_loop:
    cmp di, bx
    jb .done

    mov al, [di]
    call print_char_al

    cmp di, bx
    je .done
    dec di
    jmp .print_loop

.done:
    pop si
    pop di
    pop bx
    pop ax
    ret


; Wait for any key without printing its code.
cmd_wait:
    push ax
    push si

    mov si, msg_wait
    call print
    xor ah, ah
    int 0x16

    pop si
    pop ax
    ret

; Show current cursor position.
cmd_cursor:
    push ax
    push bx
    push dx
    push si

    mov ah, 0x03
    xor bh, bh
    int 0x10

    mov bx, dx

    mov si, msg_cursor_row
    call print
    xor ax, ax
    mov al, bh
    call print_reg

    mov si, msg_cursor_col
    call print
    xor ax, ax
    mov al, bl
    call print_reg

    pop si
    pop dx
    pop bx
    pop ax
    ret

; Show current BIOS video mode, columns and active page.
cmd_screen:
    push ax
    push bx
    push si

    mov ah, 0x0F
    int 0x10
    mov [screen_info_ax], ax
    mov [screen_info_bx], bx

    mov si, msg_screen_mode
    call print
    mov ax, [screen_info_ax]
    xor ah, ah
    call print_reg

    mov si, msg_screen_cols
    call print
    mov ax, [screen_info_ax]
    mov al, ah
    xor ah, ah
    call print_reg

    mov si, msg_screen_page
    call print
    mov ax, [screen_info_bx]
    mov al, ah
    xor ah, ah
    call print_reg

    pop si
    pop bx
    pop ax
    ret

; Approximate uptime in seconds from the low BIOS tick counter.
cmd_uptime:
    push ax
    push bx
    push cx
    push dx
    push si

    xor ah, ah
    int 0x1A

    mov ax, dx
    xor dx, dx
    mov bx, 18
    div bx

    mov si, msg_uptime
    call print
    call print_reg

    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Convert decimal number to hex: hex 4660 -> 0x1234
cmd_hex:
    push ax
    push bx
    push si

    call skip_spaces
    cmp byte [si], 0
    je .usage

    call parse_uint16
    jc .usage

    mov bx, ax
    call skip_spaces
    cmp byte [si], 0
    jne .usage

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


; Print one ASCII character by decimal code.
; Example: ascii 65 -> A
cmd_ascii:
    push ax
    push bx
    push si

    call skip_spaces
    cmp byte [si], 0
    je .usage

    call parse_uint16
    jc .usage

    cmp ax, 255
    ja .usage

    mov bx, ax
    call skip_spaces
    cmp byte [si], 0
    jne .usage

    mov si, msg_ascii
    call print
    mov al, bl
    call print_char_al
    jmp .done

.usage:
    mov si, msg_ascii_usage
    call print

.done:
    pop si
    pop bx
    pop ax
    ret

; Repeat text several times.
; Example: repeat 3 hi -> hihihi
cmd_repeat:
    push ax
    push cx
    push si

    call skip_spaces
    cmp byte [si], 0
    je .usage

    call parse_uint16
    jc .usage

    test ax, ax
    jz .done

    cmp ax, 20
    ja .too_many

    mov cx, ax
    call skip_spaces
    cmp byte [si], 0
    je .usage

.print_loop:
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



; Print saved command history.
cmd_history:
    push ax
    push bx
    push cx
    push si

    cmp byte [history_count], 0
    jne .has_items

    mov si, msg_history_empty
    call print
    jmp .done

.has_items:
    xor cx, cx
    mov cl, [history_count]

    mov al, [history_count]
    cmp al, HISTORY_SLOTS
    jb .start_from_zero

    mov al, [history_next]
    jmp .start_ready

.start_from_zero:
    xor al, al

.start_ready:
    mov [history_list_index], al
    mov bx, 1

.loop:
    mov si, msg_history_item
    call print

    mov ax, bx
    call print_reg

    mov si, msg_history_sep
    call print

    mov al, [history_list_index]
    call history_get_ptr
    call print
    call print_new_line

    inc byte [history_list_index]
    cmp byte [history_list_index], HISTORY_SLOTS
    jb .index_ok
    mov byte [history_list_index], 0

.index_ok:
    inc bx
    loop .loop

.done:
    pop si
    pop cx
    pop bx
    pop ax
    ret

; Show BIOS boot drive number from the PC info block.
cmd_boot:
    push ax
    push bx
    push es
    push si

    xor ax, ax
    mov es, ax
    xor ax, ax
    mov al, [es:PCINFO_ADDR + 2]
    mov bx, ax

    mov si, msg_boot_dec
    call print
    call print_reg

    mov si, msg_boot_hex
    call print
    mov ax, bx
    call print_hex16

    pop si
    pop es
    pop bx
    pop ax
    ret

; Show current segment registers.
cmd_segs:
    push ax
    push si

    mov si, msg_seg_cs
    call print
    push cs
    pop ax
    call print_hex16

    mov si, msg_seg_ds
    call print
    mov ax, ds
    call print_hex16

    mov si, msg_seg_es
    call print
    mov ax, es
    call print_hex16

    mov si, msg_seg_ss
    call print
    mov ax, ss
    call print_hex16

    pop si
    pop ax
    ret

; Reset video mode to 80x25 text mode.
cmd_mode3:
    push ax
    push si

    mov ax, 0x0003
    int 0x10

    mov si, msg_mode3
    call print

    pop si
    pop ax
    ret

; Switch to 80x50 text mode using the VGA 8x8 font.
cmd_mode50:
    push ax
    push bx
    push si

    mov ax, 0x0003
    int 0x10

    mov ax, 0x1112
    xor bx, bx
    int 0x10

    mov si, msg_mode50
    call print

    pop si
    pop bx
    pop ax
    ret

; Print a small banner.
cmd_banner:
    push si

    mov si, msg_banner
    call print

    pop si
    ret

; Fibonacci number: fib 10 -> 55.
; Limit is 24 so the result fits into 16 bits.
cmd_fib:
    push ax
    push bx
    push cx
    push dx
    push si

    call skip_spaces
    cmp byte [si], 0
    je .usage

    call parse_uint16
    jc .usage

    cmp ax, 24
    ja .too_big

    call skip_spaces
    cmp byte [si], 0
    jne .usage

    mov cx, ax
    xor ax, ax
    mov bx, 1

    test cx, cx
    jz .print

.loop:
    mov dx, ax
    add dx, bx
    mov ax, bx
    mov bx, dx
    loop .loop

.print:
    mov si, msg_fib
    call print
    call print_reg
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

; Show the panic screen manually.
cmd_panic:
    call debug_panic_manual
    ret

; Trigger divide-by-zero. Useful for testing the exception handler.
cmd_divzero:
    call debug_test_divzero
    ret

; Stop the CPU. Useful for emulator tests.
cmd_halt:
    mov si, msg_halt
    call print
    cli
.halt_loop:
    hlt
    jmp .halt_loop

; Skip spaces in DS:SI
skip_spaces:
    cmp byte [si], ' '
    jne .done
    inc si
    jmp skip_spaces
.done:
    ret

; > Парсинг беззнакового десятичного числа uint16
; Параметры:
;  - ds:si: строка с числом
; Вывод:
;  - ax: число
;  - si: первый символ после числа
;  - CF=0 успех, CF=1 ошибка/переполнение/нет цифр
parse_uint16:
    push bx
    push cx
    push dx

    xor ax, ax
    xor cx, cx

.next_digit:
    xor dx, dx
    mov dl, [si]
    cmp dl, '0'
    jb .finish
    cmp dl, '9'
    ja .finish

    sub dl, '0'
    push dx

    mov bx, 10
    mul bx              ; dx:ax = ax * 10
    or dx, dx
    pop dx
    jnz .error

    add ax, dx
    jc .error

    inc si
    inc cx
    jmp .next_digit

.finish:
    test cx, cx
    jz .error
    clc
    jmp .done

.error:
    stc

.done:
    pop dx
    pop cx
    pop bx
    ret

; > Печать одного символа из AL через BIOS TTY
print_char_al:
    push ax
    push bx

%ifdef KERNEL_CONSOLE
    call console_putc
%else
    mov ah, 0x0E
    xor bx, bx
    int 0x10
%endif

    pop bx
    pop ax
    ret

; > Печать двух BCD-цифр из AL
print_bcd2:
    push ax
    push bx

    mov bl, al
    mov al, bl
    shr al, 4
    and al, 0x0F
    add al, '0'
    call print_char_al

    mov al, bl
    and al, 0x0F
    add al, '0'
    call print_char_al

    pop bx
    pop ax
    ret


; Print AX as four hexadecimal digits.
print_hex16:
    push ax
    push bx
    push cx

    mov bx, ax
    mov cx, 4

.next_digit:
    rol bx, 4
    mov al, bl
    and al, 0x0F

    cmp al, 9
    jbe .digit
    add al, 'A' - 10
    jmp .print

.digit:
    add al, '0'

.print:
    call print_char_al
    loop .next_digit

    pop cx
    pop bx
    pop ax
    ret

err_unknown_cmd:      db '[!] Unknown command, write help for list of commands.', 0
err_shutdown:         db '[!] PC shutdown failed! (No APM)', 0
err_rtc:              db '[!] RTC/BIOS time service unavailable.', 0
err_calc_syntax:      db '[!] Syntax: calc <number> <+|-|*|/> <number>', 0
err_calc_overflow:    db '[!] Calculator overflow: result must fit into 16 bits.', 0
err_calc_div_zero:    db '[!] Division by zero.', 0
err_repeat_count:     db '[!] Repeat count must be 1..20.', 0
err_fib_range:        db '[!] Fib argument must be 0..24.', 0

; Сообщения
msg_help:
    db 'Realix - Help:', ENTER
    db '  [Base]', ENTER
    db '> clear/cls - Clear screen', ENTER
    db '> help - Show this manual', ENTER
    db '> echo [text] - Print [text] to console', ENTER
    db '> meminfo - Display RAM configuration', ENTER
    db '> ver - Show system version', ENTER
    db '> about - Show short system info', ENTER
    db '  [Tools]', ENTER
    db '> beep - Play BIOS bell', ENTER
    db '> date - Show RTC date', ENTER
    db '> time - Show RTC time', ENTER
    db '> calc A +|-|*|/ B - Calculate integer expression', ENTER
    db '> key - Wait key and show key codes', ENTER
    db '> ticks - Show BIOS timer ticks', ENTER
    db '> rand - Print pseudo-random number', ENTER
    db '> len [text] - Count text length', ENTER
    db '> upper [text] - Print text in upper case', ENTER
    db '> lower [text] - Print text in lower case', ENTER
    db '> reverse [text] - Print text backwards', ENTER
    db '> wait - Wait for any key', ENTER
    db '> cursor - Show cursor position', ENTER
    db '> screen - Show BIOS video mode info', ENTER
    db '> uptime - Show approximate uptime seconds', ENTER
    db '> hex [number] - Convert decimal to hex', ENTER
    db '> ascii [number] - Print ASCII character', ENTER
    db '> repeat [count] [text] - Repeat text, max 20', ENTER
    db '> history - Show saved command history', ENTER
    db '> boot - Show BIOS boot drive', ENTER
    db '> segs - Show segment registers', ENTER
    db '> mode3 - Reset 80x25 text mode', ENTER
    db '> mode50 - Switch to 80x50 text mode', ENTER
    db '> banner - Print Realix banner', ENTER
    db '> fib [0-24] - Fibonacci number', ENTER
    db '> panic - Show panic screen and halt', ENTER
    db '> divzero - Test divide error handler', ENTER
    db '  [Power]', ENTER
    db '> reboot - Reboot PC', ENTER
    db '> shutdown - Power off PC', ENTER
    db '> halt - Stop CPU', 0

msg_version:      db 'Realix v0.06-dev / Kernel16 CLI extended', 0
msg_about:        db 'Realix is a lightweight hybrid x86 OS: BIOS boot, FAT12, Kernel16 CLI.', 0
msg_time:         db 'Time: ', 0
msg_date:         db 'Date: ', 0
msg_calc_usage:   db 'Usage: calc <number> <+|-|*|/> <number>', 0
msg_calc_result:  db 'Result: ', 0
msg_key_wait:     db 'Press any key...', ENTER, 0
msg_key_ascii:    db 'ASCII: ', 0
msg_key_scan:     db ', scan: ', 0
msg_ticks_hi:     db 'Ticks high: ', 0
msg_ticks_low:    db ', low: ', 0
msg_rand:         db 'Random: ', 0
msg_len:          db 'Length: ', 0
msg_wait:         db 'Press any key...', 0
msg_cursor_row:   db 'Cursor row: ', 0
msg_cursor_col:   db ', column: ', 0
msg_screen_mode:  db 'Video mode: ', 0
msg_screen_cols:  db ', columns: ', 0
msg_screen_page:  db ', page: ', 0
msg_uptime:       db 'Uptime seconds low: ', 0
msg_hex:          db 'Hex: 0x', 0
msg_hex_usage:    db 'Usage: hex <number>', 0
msg_ascii:        db 'Char: ', 0
msg_ascii_usage:  db 'Usage: ascii <0-255>', 0
msg_repeat_usage: db 'Usage: repeat <1-20> <text>', 0
msg_history_empty: db 'History is empty.', 0
msg_history_item:  db '#', 0
msg_history_sep:   db ': ', 0
msg_boot_dec:     db 'Boot drive: ', 0
msg_boot_hex:     db ', hex: 0x', 0
msg_seg_cs:       db 'CS=0x', 0
msg_seg_ds:       db ' DS=0x', 0
msg_seg_es:       db ' ES=0x', 0
msg_seg_ss:       db ' SS=0x', 0
msg_mode3:        db 'Text mode 80x25 restored.', 0
msg_mode50:       db 'Text mode 80x50 enabled.', 0
msg_banner:
    db '================', ENTER
    db ' Realix Kernel16', ENTER
    db '================', 0
msg_fib:          db 'Fib: ', 0
msg_fib_usage:    db 'Usage: fib <0-24>', 0
msg_halt:         db 'CPU halted.', 0

; Variables
calc_left:  dw 0
calc_op:    db 0
rand_state:      dw 0xACE1
screen_info_ax:  dw 0
screen_info_bx:  dw 0
history_list_index: db 0

; Буфер ввода
input_str: times 64 db 0
