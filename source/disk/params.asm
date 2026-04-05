; © Realix > Disk Params
; (05.04.26) v0.04
; ================
; Зависимости: disk/read.asm, error_handler (функция)

; > Чтение параметров диска
; Вывод:
;  - cx: bpb_sectors_per_track
;  - dh: bpb_heads
;  - Ошибка: переходит к disk/read/read_error (noreturn)
disk_params:
    push ax

    ; Считывание параметров диска
    push es

    mov ah, 08h   ; Режим получения параметров диска
    int 0x13
    jc read_error
    
    pop es

    ; Обновляем кол-во секторов на дорожку и кол-во голов
    and cl, 0x3F  ; Убираем верхние 2 бита
    xor ch, ch
    inc dh

    pop ax
    ret