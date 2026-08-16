; © Realix > Keyboard
; (16.08.26) v0.12
; ================
; ❗️ В режиме KEYBOARD_MINIMAL (Bootix) доступно только `wait_key`

; > Ожидание нажатия клавиши (Блокирующее)
; Вывод:
;  - ah: Scan код нажатой клавиши
;  - al: ASCII код нажатой клавиши
wait_key:
    mov ah, 0h
    int 0x16

    ret

; > Блок "Расширенная клавиатура" (Не для Bootix)
%ifndef KEYBOARD_MINIMAL

; > Проверка нажатия клавиши из буфера (Неблокирующее)
; Вывод:
;  - ah: Scan код нажатой клавиши
;  - al: ASCII код нажатой клавиши
;  - ZF (Zero Flag): 0 (Клавиша нажата), 1 (Буфер пуст)
wait_key_nb:
    mov ah, 1h
    int 0x16

    ret

%endif
