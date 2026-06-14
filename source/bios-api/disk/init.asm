; © Realix > Disk Initialization
; (13.06.26) v0.06
; ================

; Защита от повторного включения
%ifndef DISK_INIT
%define DISK_INIT

; > Инициализация и получение параметров диска (Вызывается 1 раз при подключении)
; Параметры:
;   - dl: номер диска
; Вывод:
;  - cx: sectors_per_track
;  - dh: heads
;  - CF (Carry Flag): 0 — успех, 1 — ошибка чтения
;  - Заполняет переменные disk_spt и disk_heads
disk_init:
    push ax

    ; Считывание параметров диска
    push es
    mov ah, 8h
    int 0x13
    pop es
    jc .done    ; Выходим из функции с CF (Ошибка)

    ; Форматируем кол-во секторов на дорожку (Убираем байты цилиндра)
    and cl, 0x3F
    xor ch, ch
    mov [disk_spt], cx

    ; Форматируем кол-во голов (Увеличиваем на 1, т.к. счёт с 0)
    inc dh

    ; Сохраняем значение кол-ва голов
    xor ax, ax
    mov al, dh
    mov [disk_heads], ax

.done:
    pop ax
    ret

; Переменные геометрии текущего диска
disk_spt:   dw 0
disk_heads: dw 0

%endif