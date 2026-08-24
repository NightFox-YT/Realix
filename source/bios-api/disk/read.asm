; © Realix > Disk: Read
; (13.08.26) v0.12
; ================
; ❗️ Зависимости: error_handler (внешний обработчик), bios-api/disk/init (переменные)

; ❗️ Требуется инициализация диска через `bios_disk_init`
%include "bios-api/disk/init.asm"


; > Чтение секторов с диска
; ❗️ Номер диска берётся из `bios-api/disk/init/curr_drive_num`
;    При неудачном завершении использует error_handler (внешний обработчик)
; Параметры:
;  - ax: LBA
;  - cl: кол-во секторов для чтения (до 128)
;  - es:bx: адрес памяти для записи данных
bios_disk_read:
    push ax
    push cx
    push dx
    push di

    ; Получаем номер текущего диска
    mov dl, [curr_drive_num]

    push cx          ; *Сохраняем кол-во секторов (cl)
    call lba_to_chs
    pop ax           ; *Восстанавливаем кол-во секторов (cl -> al)

    mov ah, 02h      ; Функция BIOS: Чтение секторов
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
    pop di
    pop dx
    pop cx
    pop ax

    ; Все попытки исчерпаны
    jmp read_error

.done:
    popa

    pop di
    pop dx
    pop cx
    pop ax
    ret


; > Перевод LBA адреса в CHS адрес
; ❗️ Геометрия диска берётся из `bios-api/disk/init`
; Параметры:
;  - ax: LBA
; Вывод:
;  - cx [bits 0-5]: сектор
;  - cx [bits 6-15]: цилиндр
;  - dh: голова
lba_to_chs:
    push ax
    push dx

    ; Вычисление номера сектора (LBA / SectorsPerTrack) + 1
    xor dx, dx
    div word [curr_drive_spt]  ; ax = LBA / SPT, dx = LBA % SPT
    inc dx                     ; Сектора в CHS нумеруются с 1
    mov cx, dx                 ; Сохраняем номер сектора (cx)

    ; Вычисление номеров:
    ; > ax (Цилиндр) = (LBA / SPT) / Heads
    ; > dx (Голова)  = (LBA / SPT) % Heads
    xor dx, dx
    div word [curr_drive_heads]
    mov dh, dl  ; Сохраняем номер головы (dh)

    ; Упаковка цилиндра и сектора в cx для int 0x13
    mov ch, al  ; Сохраняем [bits 8-15] цилиндра в ch
    shl ah, 6   ; Сдвигаем старшие 2 бита цилиндра
    or cl, ah   ; Перемещаем верхние 2 бита [bits 6-7] в cl

    pop ax      ; *Восстанавливаем ax <- оригинальный dx
    mov dl, al  ; Возвращаем номер диска на место в dl
    pop ax      ; *Восстанавливаем LBA (оригинальный ax)

    ret


; > Сброс контроллера диска (*Доп. функция)
; Параметры:
;  - dl: номер диска
disk_reset:
    pusha

    xor ax, ax
    stc           ; Установка Carry Flag (Некоторые BIOS не устанавливают)
    int 0x13
    jc read_error

    popa
    ret


; > Критическая ошибка чтения
read_error:
    mov si, err_read_failed
    jmp error_handler

err_read_failed: db '[#] Read failed!', 0
