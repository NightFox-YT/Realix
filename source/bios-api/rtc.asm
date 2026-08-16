; © Realix > RTC
; (15.08.26) v0.12
; ================

; > Получение текущего значения прошедших тиков
; Вывод:
;  - eax: 32-битное значение прошедших тиков
get_ticks_value:
    push ecx

    mov ah, 0h
    int 0x1A

    shl ecx, 16
    movzx eax, dx
    or eax, ecx

    pop ecx
    ret


; > Сохранение текущего значения прошедших тиков
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


; > Получение даты
; Вывод:
;  - ch, cl: век, год (BCD)
;  - dh, dl: месяц, день (BCD)
;  - CF (Carry Flag): 0 (Успех), 1 (Неудача)
get_date:
    push ax

    mov ah, 04h
    int 0x1A
    
    pop ax
    ret


; Переменные
init_tick: dd 0