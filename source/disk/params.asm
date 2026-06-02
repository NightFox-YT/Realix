; © Realix > Disk Params
; (02.06.26) v0.05
; ================
; Зависимости: disk/read.asm (+error_handler)

; > Чтение параметров диска
; Вывод:
;  - cx: bpb_sectors_per_track
;  - dh: bpb_heads
;  - Ошибка: переходит к disk/read/read_error (noreturn)
disk_params:
    push ax

    ; Считывание параметров диска
    push es
    mov ah, 8h
    int 0x13
    jc read_error
    pop es

    ; Форматируем кол-во секторов на дорожку (Убираем байты цилиндра)
    and cl, 0x3F
    xor ch, ch

    ; Форматируем кол-во голов (Увеличиваем на 1)
    inc dh

    pop ax
    ret