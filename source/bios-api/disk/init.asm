; © Realix > Disk: Initialization
; (13.08.26) v0.12
; ================

; Защита от повторного включения модуля
%ifndef BIOS_DISK_INIT
%define BIOS_DISK_INIT


; > Инициализация и получение параметров диска через BIOS
; (Вызывается при переключении/инициализации диска)
; Параметры:
;  - dl: номер диска
; Вывод:
;  - cx: sectors_per_track
;  - dh: heads
;  - dl: number of hard hisk drives
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка чтения)
;  - Заполняет переменные curr_drive_num, curr_drive_spt и curr_drive_heads
bios_disk_init:
    push ax
    push bx
    push di
    push es

    ; Запоминаем номер инициализированного диска
    mov [curr_drive_num], dl

    ; Считывание параметров диска (С защитой от бага на некоторых BIOS)
    ; > es:di - указатель на таблицу параметров дискеты, => обнуляем
    xor di, di
    mov es, di

    mov ah, 08h
    int 0x13
    jc .return  ; Выходим из функции (Ошибка)

    ; Форматируем кол-во секторов на дорожку (Убираем байты цилиндра)
    and cl, 0x3F
    xor ch, ch

    ; Форматируем кол-во голов (Увеличиваем на 1, счёт с 0)
    inc dh

    ; Сохраняем значения в переменные
    movzx ax, dh
    mov [curr_drive_heads], ax
    mov [curr_drive_spt], cx

.return:
    pop es
    pop di
    pop bx
    pop ax
    ret


; Переменные текущего диска (Номер и геометрия)
curr_drive_num:   db 0
curr_drive_spt:   dw 0
curr_drive_heads: dw 0

%endif
