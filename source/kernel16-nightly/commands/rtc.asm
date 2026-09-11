; © Realix > Commands: RTC (time / date)
; (28.07.26) v0.11
; ================
; ❗️ Зависимости: kernel16-nightly/io: print, print_ctrl, print_reg (print_bcd2)

; > Команда вывода времени (RTC)
cmd_time:
    push ax
    push cx
    push dx
    push si

    ; Чтение времени: ch - часы, cl - минуты, dh - секунды (BCD)
    mov ah, 0x02
    int 0x1A
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
    mov ah, 0x04
    int 0x1A
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
