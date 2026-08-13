; © Realix > Disk: Initialization
; (27.07.26) v0.1
; ================

; Защита от повторного включения модуля
%ifndef DISK_INIT
%define DISK_INIT

; > Инициализация и получение параметров диска (Вызывается на старте)
; Параметры:
;  - dl: номер диска
; Вывод:
;  - cx: sectors_per_track
;  - dh: heads
;  - dl: num of hard hisk drives
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка чтения)
;  - Заполняет переменные disk_current_drive, disk_spt и disk_heads
disk_init:
    push ax
    push bx
    push di

    ; Запоминаем номер диска до вызова BIOS
    mov [disk_current_drive], dl

    ; Считывание параметров диска (С защитой от бага на некоторых BIOS)
    push es
    xor di, di
    mov es, di

    mov ah, 8h
    int 0x13
    
    pop es
    jc .done  ; Выходим из функции (Ошибка)

    ; Форматируем кол-во секторов на дорожку (Убираем байты цилиндра)
    and cl, 0x3F
    xor ch, ch

    ; Форматируем кол-во голов (Увеличиваем на 1, счёт с 0)
    inc dh

    ; Сохраняем значения в переменные
    movzx ax, dh
    mov [disk_heads], ax
    mov [disk_spt], cx

.done:
    pop di
    pop bx
    pop ax
    ret

; Переменные текущего диска (Номер и геометрия)
disk_current_drive: db 0
disk_spt:           dw 0
disk_heads:         dw 0

%endif