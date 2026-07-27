; © Realix > Disk Read
; (27.07.26) v0.1
; ================
; ❗️ Зависимости: error_handler (внешний обработчик)

; ❗️ Требуется инициализация диска через `disk_init`
%include "bios-api/disk/init.asm"

; > Чтение секторов с диска
; ❗️ Номер диска берётся из disk_current_drive (заполняет `disk_init`)
; Параметры:
;  - ax: LBA
;  - cl: кол-во секторов для чтения (до 128)
;  - es:bx: адрес памяти для записи данных
disk_read:
    push cx
    push dx
    push di
    push ax

    ; Номер текущего диска для BIOS
    mov dl, [disk_current_drive]

    push cx          ; *Сохраняем кол-во секторов (cl)
    call .lba_to_chs
    pop ax           ; *Восстанавливаем кол-во секторов (cl > al)

    mov ah, 2h       ; Функция BIOS: Чтение секторов
    mov di, 3        ; Кол-во попыток чтения

.retry:
    pusha     ; Сохранение регистров (BIOS может их испортить)
    stc       ; Установка Carry Flag (Некоторые BIOS не устанавливают)
    int 0x13
    jnc .done

    ; Если "Ошибка", сбрасываем контроллер диска
    popa
    call disk_reset

    ; Переход к след. попытке
    dec di
    jnz .retry

.fail:
    pop ax
    pop di
    pop dx
    pop cx

    ; Все попытки исчерпаны
    jmp read_error

.done:
    popa

    pop ax
    pop di
    pop dx
    pop cx
    ret


; > Перевод LBA адреса в CHS адрес
; Параметры:
;  - ax: LBA
; Вывод:
;  - cx [bits 0-5]: сектор
;  - cx [bits 6-15]: цилиндр
;  - dh: номер головы
.lba_to_chs:
    push ax
    push dx

    ; Вычисление номера сектора (LBA / SectorsPerTrack) + 1
    xor dx, dx
    div word [disk_spt]  ; ax = LBA / SPT, dx = LBA % SPT
    inc dx               ; Сектора в CHS нумеруются с 1
    mov cx, dx           ; Сохраняем номер сектора (cx)

    ; Вычисление номеров:
    ; > ax (Цилиндр) = (LBA / SPT) / Heads
    ; > dx (Голова)  = (LBA / SPT) % Heads
    xor dx, dx
    div word [disk_heads]
    mov dh, dl             ; Сохраняем номер головы (dh)

    ; Упаковка цилиндра и сектора в cx для int 0x13
    mov ch, al  ; Сохраняем [bits 8-15] цилиндра в ch
    shl ah, 6   ; Сдвигаем старшие 2 бита цилиндра
    or cl, ah   ; Перемещаем верхние 2 бита [bits 6-7] в cl

    pop ax      ; *Восстанавливаем ax ← оригинальный dx
    mov dl, al  ; Возвращаем номер диска на место в dl
    pop ax      ; *Восстанавливаем оригинальный ax (LBA)

    ret


; > Сброс контроллера диска
; ❗️ Номер диска берётся из disk_current_drive (заполняет `disk_init`)
disk_reset:
    pusha

    mov dl, [disk_current_drive]
    
    ; Сброс контроллера диска
    xor ax, ax  ; Функция BIOS: Сброс дискового контроллера
    stc         ; Установка Carry Flag (Некоторые BIOS не устанавливают)
    int 0x13
    jc read_error

    popa
    ret


; > Ошибки
read_error:
    mov si, err_read_failed
    jmp error_handler

err_read_failed: db '[!] Read failed!', 0