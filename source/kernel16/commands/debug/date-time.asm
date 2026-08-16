; © Realix > Commands: RTC (time / date)
; (28.07.26) v0.11
; ================
; ❗️ Зависимости: bios-api/io/*, bios-api/rtc

; > Команда вывода времени (RTC)
cmd_time:
    push ax
    push cx
    push dx
    push si

    ; Чтение времени: ch - часы, cl - минуты, dh - секунды (BCD)
    call get_time
    jc .fail

    ; Выводим сообщение о времени
    mov si, msg_time
    call print

    ; "ЧЧ:ММ:СС"
    mov al, ch
    call print_bcd2
    mov al, ':'
    call print_char
    mov al, cl
    call print_bcd2
    mov al, ':'
    call print_char
    mov al, dh
    call print_bcd2
    jmp .done

.fail:
    ; Ошибка 9
    mov si, err_time
    call print

.done:
    pop si
    pop dx
    pop cx
    pop ax
    ret

; > Команда вывода даты (RTC)
cmd_date:
    push ax
    push cx
    push dx
    push si

    ; Чтение даты: ch - век, cl - год, dh - месяц, dl - день (BCD)
    call get_date
    jc .fail

    ; Выводим сообщение о дате
    mov si, msg_date
    call print

    ; "ДД.ММ."
    mov al, dl
    call print_bcd2
    mov al, '.'
    call print_char
    mov al, dh
    call print_bcd2
    mov al, '.'
    call print_char

    ; "ГГГГ": Год состоит из двух BCD-байтов века и года
    mov al, ch
    call print_bcd2
    mov al, cl
    call print_bcd2
    jmp .done

.fail:
    ; Ошибка 10
    mov si, err_date
    call print

.done:
    pop si
    pop dx
    pop cx
    pop ax
    ret

; Строки
msg_time: db 'Time: ', 0
msg_date: db 'Date: ', 0

; Сообщения об ошибках
err_time: db '[!] E9: Failed to read RTC time.', 0
err_date: db '[!] E10: Failed to read RTC date.', 0
