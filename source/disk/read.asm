; © Realix > Read
; (22.03.26) v0.03
; ================
; Зависимости: kernel/print.asm, kernel/print_reg.asm

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
    ; Установка фрейма функции
    push bp
    mov bp, sp

    push bx
    push cx
    push dx
    push di
    push ax

    push cx         ; Сохраняем кол-во секторов (cl)
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

    ; Переход к след. попытке
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

    pop ax           ; *Восстанавливаем LBA (al)
    call print_reg

    mov si, new_line ; "Enter"
    call print

    pop di
    pop dx
    pop cx
    pop bx

    pop bp
    ret 4

; > Перевод LBA адреса в CHS адрес
; ! NOTE: Фрейм функции отсутствует, т.к. функция локальная для disk_read
; Параметры:
;  - [bp+6]: bpb_sectors_per_track  (секторов на дорожку)
;  - [bp+4]: bpb_heads (кол-во голов)
;  - ax: LBA
; Вывод:
;  - cx [bits 0-5]: сектор
;  - cx [bits 6-15]: цилиндр
;  - dh: голова
lba_to_chs:
    push ax
    push dx

    ; Вычисляем номер сектора (LBA / SectorsPerTrack)
    xor dx, dx
    div word [bp+6] ; ax = LBA / SPT, dx = LBA % SPT
    inc dx          ; Сектора нумеруются с 1
    mov cx, dx      ; Сохраняем номер сектора (cx)

    ; Вычисляем номер цилиндра, головы ((LBA / SectorsPerTrack) / Heads)
    xor dx, dx
    div word [bp+4] ; (Цилиндр) ax = (LBA / SPT) / Heads, (Голова) dx = (LBA / SPT) % Heads
    mov dh, dl      ; Сохраняем номер головы в dh

    ; Формируем cx для INT 0x13
    mov ch, al ; Сохраняем [bits 8-15] циллиндра в ch
    shl ah, 6  ; Оставляем 2 старших бита
    or cl, ah  ; Перемещаем верхние 2 бита [bits 6-8] в cl

    pop ax     ; *Восстанавливаем оригинальный dx → ax
    mov dl, al ; Восстанавливаем dl
    pop ax     ; *Восстанавливаем ax

    ret

; > Сброс контроллера диска
; Параметры:
;  - dl: номер диска
disk_reset:
    pusha

    stc ; Установка carry flag (BIOS может не устанавливать)

    ; Режим сброса диска
    mov ah, 0
    int 0x13
    jc read_error

    popa
    ret

; > Ошибки
read_error:
    mov si, err_read_failed
    jmp error_handler