; © Realix > Read
; (03.04.26) v0.04
; ================
; Зависимости: error_handler (функция)

; > Чтение секторов с диска
; Параметры (Стек):
;  - [bp+4]: bpb_heads             (кол-во голов)
;  - [bp+6]: bpb_sectors_per_track (секторов на дорожку)
; Параметры (Регистры):
;  - ax: LBA
;  - cl: кол-во секторов для чтения (до 128)
;  - dl: номер диска
;  - es:bx: адрес памяти для записи данных
; Вывод:
;  - Успех: возвращает управление
;  - Ошибка: переходит к read_error (noreturn)
disk_read:
    ; Установка фрейма функции
    push bp
    mov bp, sp

    push bx
    push cx
    push dx
    push di
    push ax

    push cx          ; Сохраняем кол-во секторов (cl)
    call .lba_to_chs
    pop ax           ; Восстанавливаем кол-во секторов (cl > al)

    mov ah, 02h ; Режим чтения секторов
    mov di, 3   ; Кол-во попыток чтения

.retry:
    pusha    ; Сохранение регистров (BIOS может изменить)
    stc      ; Установка Carry Flag (Некоторые BIOS не устанавливают)
    int 0x13
    jnc .done

    ; Ошибка чтения
    popa
    call disk_reset

    ; Переход к след. попытке (dec автоматически устанавливает ZF)
    dec di
    jnz .retry

; Все попытки исчерпаны
.fail:
    jmp read_error

.done:
    popa

    pop ax
    pop di
    pop dx
    pop cx
    pop bx

    ; Очистка аргументов стека
    pop bp
    ret 4

; > Перевод LBA адреса в CHS адрес (❗Локальная функция)
; Параметры (наследует фрейм disk_read):
;  - [bp+6]: bpb_sectors_per_track
;  - [bp+4]: bpb_heads
;  - ax: LBA
; Вывод:
;  - cx [bits 0-5]: сектор
;  - cx [bits 6-15]: цилиндр
;  - dh: голова
.lba_to_chs:
    push ax
    push dx

    ; Вычисляем номер сектора (LBA / SectorsPerTrack) + 1
    xor dx, dx
    div word [bp+6] ; ax = LBA / SPT, dx = LBA % SPT
    inc dx          ; Сектора нумеруются с 1
    mov cx, dx      ; Сохраняем номер сектора (cx)

    ; Вычисляем номера:
    ; - (ax) Цилиндра: (LBA / SPT) / Heads
    ; - (dx) Головы:   (LBA / SPT) % Heads
    xor dx, dx
    div word [bp+4]
    mov dh, dl      ; Сохраняем номер головы (dh)

    ; Формируем cx для INT 0x13
    mov ch, al ; Сохраняем [bits 8-15] циллиндра в ch
    shl ah, 6  ; Оставляем 2 старших бита
    or cl, ah  ; Перемещаем верхние 2 бита [bits 6-7] в cl

    pop ax     ; *Восстанавливаем ax ← оригинальный dx
    mov dl, al ; Восстанавливаем dl (номер диска)
    pop ax     ; *Восстанавливаем ax (LBA)

    ret

; > Сброс контроллера диска
; Параметры:
;  - dl: номер диска
; Вывод:
;  - Успех: возвращает управление
;  - Ошибка: переходит к read_error (noreturn)
disk_reset:
    pusha
    stc   ; Установка Carry Flag (Некоторые BIOS не устанавливают)

    ; Режим сброса диска
    mov ah, 0
    int 0x13
    jc read_error

    popa
    ret

; > Чтение параметров диска
; Вывод:
;  - cx: bpb_sectors_per_track
;  - dh: bpb_heads
;  - Ошибка: переходит к read_error (noreturn)
disk_params:
    push ax

    ; Считывание параметров диска (Секторов на дорожку и кол-во голов)
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

; > Ошибка чтения
; Вывод:
;  - Вызывает error_handler (noreturn)
read_error:
    mov si, err_read_failed
    jmp error_handler

; Сообщения
err_read_failed: db '[!] Read failed!', 0