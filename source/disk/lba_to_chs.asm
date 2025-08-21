; Realix > LBA to CHS
; (C) v0.03 | 19.08.25
; ===================

; [Disk] Перевод LBA адреса в CHS адрес
; Параметры:
;  - ax: LBA
; Вывод:
;  - cx [bits 0-5]: сектор
;  - cx [bits 6-15]: цилиндр
;  - dh: голова
lba_to_chs:
    push ax
    push dx

    ; Вычисляем номер сектора
    xor dx, dx 
    div word [bpb_sectors_per_track] ; ax = LBA / SectorsPerTrack, dx = LBA % SectorsPerTrack
    inc dx                           ; (Сектор) dx = (LBA % SectorsPerTrack) + 1
    mov cx, dx                       ; Сохраняем номер сектора в cx

    ; Вычисляем номер головы, цилиндра
    xor dx, dx
    div word [bpb_heads] ; (Цилиндр) ax = (LBA / SectorsPerTrack) / Heads, (Голова) dx = (LBA / SectorsPerTrack) % Heads
    mov dh, dl           ; Сохраняем номер головы в dh

    ; Формируем cx для прерывания BIOS
    mov ch, al ; Сохраняем [bits 8-15] циллиндра в ch
    shl ah, 6  ; Оставляем 2 старших бита
    or cl, ah  ; Перемещаем верхние 2 бита [bits 6-8] в cl

    pop ax     ; *Восстановление значение dx в ax
    mov dl, al ; Восстановление только dl
    pop ax

    ret