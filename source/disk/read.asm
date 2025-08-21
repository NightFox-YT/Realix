; Realix > Read sectors
; (C) v0.03 | 19.08.25
; =====================

; [Disk] Чтение секторов с диска
; Параметры:
;  - ax: LBA
;  - cl: кол-во секторов для чтения (до 128)
;  - dl: номер диска
;  - es:bx: адрес памяти, где сохранить прочитанные данные
disk_read:
    push bx
    push cx
    push dx
    push di
    push ax

    push cx         ; *Временно сохраняем cl (кол-во секторов для чтения)
    call lba_to_chs
    
    pop ax      ; *Восстанавливаем cl в al (кол-во секторов для чтения)
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

.fail:
    ; Все попытки исчерпаны
    jmp read_error

.done:
    popa

    ; Вывод о успешном прочтении
    mov si, msg_read_success ; "[LOG] Read ..."
    call print

    xor ah, ah
    call print_reg ; Выводим число секторов
    add si, 12     ; "sectors, LBA: ..." + Пропускаем символы, которые уже вывели.
    call print

    pop ax         ; *Восстанавливаем LBA
    
    call print_reg ; Выводим LBA
    add si, 16     ; ENTER + Пропускаем символы, которые уже вывели.
    call print

    pop di
    pop dx
    pop cx
    pop bx
    ret

; [Disk] Сбрасываем контроллер диска
; Параметры:
;  - dl: номер диска
disk_reset:
    pusha
    mov ah, 0     ; Режима сброса диска
    stc           ; Установка carry flag (BIOS может не устанавливать)
    int 0x13
    jc read_error ; Если carry flag != 0, диск сломался...
    popa
    ret

; [Disk] Ошибки
read_error:
    mov si, err_read_failed ; "Чтение с диска не удалось"
    call print
    jmp error_handler