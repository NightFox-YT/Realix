; © Realix > LBA to CHS
; (21.03.26) v0.03
; ================

; > Перевод LBA адреса в CHS адрес
; Параметры:
;  - [bp+6]: bpb_sectors_per_track  (секторов на дорожку)
;  - [bp+4]: bpb_heads (кол-во голов)
;  - ax: LBA
; Вывод:
;  - cx [bits 0-5]: сектор
;  - cx [bits 6-15]: цилиндр
;  - dh: голова
lba_to_chs:
    push bp
    mov bp, sp

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
 
    pop bp
    ret 4