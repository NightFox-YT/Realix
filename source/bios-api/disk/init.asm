; © Realix > Disk: Initialization
; (13.08.26) v0.12
; ================

; Защита от повторного включения модуля
%ifndef BIOS_DISK_INIT
%define BIOS_DISK_INIT

; > Инициализация и получение параметров диска через BIOS (Вызывается на старте)
; Параметры:
;  - dl: номер диска
; Вывод:
;  - cx: sectors_per_track
;  - dh: heads
;  - dl: number of hard hisk drives
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка чтения)
;  - Заполняет переменные curr_drive_num, curr_disk_spt и curr_disk_heads
bios_disk_init:
    push ax
    push bx
    push di

    ; Запоминаем номер инициализированного диска
    mov [curr_drive_num], dl

    ; Считывание параметров диска (С защитой от бага на некоторых BIOS)
    ; > es:di - указатель на таблицу параметров дискеты, => обнуляем
    push es
    xor di, di
    mov es, di

    mov ah, 8h
    int 0x13
    pop es
    jc .return  ; Выходим из функции (Ошибка)

    ; Форматируем кол-во секторов на дорожку (Убираем байты цилиндра)
    and cl, 0x3F
    xor ch, ch

    ; Форматируем кол-во голов (Увеличиваем на 1, счёт с 0)
    inc dh

    ; Сохраняем значения в переменные
    movzx ax, dh
    mov [curr_disk_heads], ax
    mov [curr_disk_spt], cx

.return:
    pop di
    pop bx
    pop ax
    ret

; Переменные текущего диска (Номер и геометрия)
curr_drive_num:  db 0
curr_disk_spt:   dw 0
curr_disk_heads: dw 0

%endif