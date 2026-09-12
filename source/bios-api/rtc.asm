; © Realix > RTC
; (15.08.26) v0.12
; ================


; > Получение значения текущего кол-ва прошедших тиков
; Вывод:
;  - eax: 32-битное значение прошедших тиков
get_ticks_value:    
    push ecx
    push dx

    ; Функция BIOS
    mov ah, 0h
    int 0x1A

    ; Значение из cx:dx пишем в eax
    shl ecx, 16
    movzx eax, dx
    or eax, ecx

    pop dx
    pop ecx
    ret


; > Сохранение значения текущего кол-во прошедших тиков
; Вывод:
;  - Заполнение переменной `init_tick`
save_ticks_value:
    push eax

    call get_ticks_value
    mov dword [init_tick], eax

    pop eax
    ret


; > Получение времени
; Вывод:
;  - ch, cl, dh: часы, минуты, секунды (BCD)
;  - CF (Carry Flag): 0 (Успех), 1 (Неудача)
get_time:
    push ax

    mov ah, 02h
    int 0x1A
    
    pop ax
    ret


; > Получение текущей даты
; Вывод:
;  - ch, cl: век, год (BCD)
;  - dh, dl: месяц, день (BCD)
;  - CF (Carry Flag): 0 (Успех), 1 (Неудача)
get_date:
    push ax

    ; Функция BIOS
    mov ah, 04h
    int 0x1A
    
    pop ax
    ret


; Переменные
init_tick: dd 0