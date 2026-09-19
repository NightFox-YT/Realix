; © Realix > Command: Uptime
; (06.08.26) v0.11
; ================
; ❗️ Зависимости: bios-api/rtc (last_tick_*), 


; > Вычисление текущего аптайма в секундах при помощи таймера BIOS
cmd_uptime:
    push eax
    push ebx
    push si

    ; Вывод заголовка сообщения команды
    mov si, msg_uptime
    call print

    ; Получаем текущее кол-во тиков и считаем сколько прошло (в eax)
    call get_ticks_value
    sub eax, dword [init_tick]

    ; Перевод тиков в секунды
    mov ebx, 10
    mul ebx
    mov ebx, 182
    div ebx
    
    ; Перевод в минуты с сохранением секунд
    xor edx, edx
    mov ebx, 60
    div ebx
    push edx

    ; Перевод в часы с сохранением минут
    xor edx, edx
    mov ebx, 60
    div ebx
    push edx

    ; Перевод в дни с сохранением часов и дней
    xor edx, edx
    mov ebx, 24
    div ebx
    push edx
    push eax

    ; Устанавливаем флаг
    mov byte [.printed_flag], 0

    ; Проверяем кол-во дней
    pop eax
    test eax, eax
    jz .skip_days

.print_days_value:
    ; Выводим кол-во дней
    call print_dec32
    mov si, str_days
    call print

    mov si, str_sep
    call print

    ; Вывели что-то не нулевое ;)
    mov byte [.printed_flag], 1

.skip_days:
    ; Проверяем флаг
    pop eax
    cmp byte [.printed_flag], 0
    jnz .print_hours_value

    ; Проверяем кол-во часов
    test eax, eax
    jz .skip_hours

.print_hours_value:
    ; Выводим кол-во часов
    call print_dec32
    mov si, str_hours
    call print

    mov si, str_sep
    call print

    ; Вывели что-то не нулевое ;)
    mov byte [.printed_flag], 1

.skip_hours:
    ; Проверяем флаг
    pop eax
    cmp byte [.printed_flag], 0
    jnz .print_minutes_value

    ; Проверяем кол-во минут
    test eax, eax
    jz .print_seconds_value

.print_minutes_value:
    ; Выводим кол-во минут
    call print_dec32
    mov si, str_minutes
    call print

    mov si, str_sep
    call print

    ; Вывели что-то не нулевое ;)
    mov byte [.printed_flag], 1

.print_seconds_value:
    ; Выводим кол-во секунд
    pop eax
    call print_dec32
    mov si, str_seconds
    call print

.done:
    pop si
    pop ebx
    pop eax
    ret

.printed_flag: db 0

; Сообщение
msg_uptime: db 'Uptime: ', 0

; Вспомогательные строки
str_seconds: db ' seconds', 0
str_minutes: db ' minutes', 0
str_hours:   db ' hours', 0
str_days:    db ' days', 0
str_sep:     db ', ', 0
