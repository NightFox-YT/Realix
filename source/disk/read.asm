; © Realix > Read
; (21.03.26) v0.03
; ================

; > Чтение секторов с диска
; Параметры (Стек):
;  - [bp+4]: bpb_heads             (кол-во голов)
;  - [bp+6]: bpb_sectors_per_track (секторов на дорожку)
; Параметры (Регистры):
;  - ax: LBA
;  - cl: кол-во секторов для чтения (до 128)
;  - dl: номер диска
;  - es:bx: адрес памяти, где сохранить прочитанные данные
disk_read:
    push bp
    mov bp, sp

    push bx
    push cx
    push dx
    push di
    push ax

    push cx          ; Сохраняем кол-во секторов (cl)
    push word [bp+6]
    push word [bp+4]
    call lba_to_chs
    
    pop ax      ; Восстанавливаем кол-во секторов (cl > al)
    mov ah, 02h ; Режим чтения секторов
    mov di, 3   ; Кол-во попыток чтения

.retry:
    pusha    ; Сохранение всех регистров (BIOS может изменить)
    stc      ; Установка carry flag (BIOS может не устанавливать)
    int 0x13
    jnc .done

    ; Ошибка чтения
    popa
    call disk_reset

    ; Уменьшаем кол-во попыток
    dec di
    test di, di
    jnz .retry

; Все попытки исчерпаны
.fail:
    jmp read_error

.done:
    popa

    ; Вывод в консоль
    mov si, msg_read_ok ; "[LOG] Read OK: LBA "
    call print

    pop ax              ; *Восстанавливаем LBA (al)
    call print_reg

    mov si, new_line    ; "Enter"
    call print

    pop di
    pop dx
    pop cx
    pop bx
    pop bp
    ret 4

; > Сброс контроллер диска
; Параметры:
;  - dl: номер диска
disk_reset:
    pusha
    mov ah, 0     ; Режим сброса диска
    stc           ; Установка carry flag (BIOS может не устанавливать)
    int 0x13
    jc read_error
    popa
    ret

; > Ошибки
read_error:
    mov si, err_read_failed ; "[!] Read failed!"
    call print
    jmp error_handler