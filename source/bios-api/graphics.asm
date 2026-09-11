; © Realix > Graphics (VGA)
; (16.08.26) v0.12
; ================

; Основные константы
%include 'shared/config.asm'

; Константы экрана VGA
VGA_WIDTH   equ 320
VGA_HEIGHT  equ 200
VGA_SEGMENT equ 0xA000


; > Включение видеорежима (320x200, 256 цветов)
enable_vga_videomode:
    push ax

    ; Функция BIOS: Установка режима (ah = 0, al = 13h)
    mov ax, VIDEO_MODE_320x200
    int 0x10

    pop ax
    ret


; > Восстановление стандартной 16-цветной EGA/VGA палитры (регистры DAC
; 0-15). mode 13h гарантированно приходит с этой палитрой по умолчанию -
; вызов здесь просто явно подтверждает то, что и так должно быть, а не
; полагается на умолчание (см. drivers::vga::Color в kernel32 - всегда
; пишет именно индекс 0-15, никогда RGB напрямую, так что от палитры
; напрямую зависит, какие цвета реально видны)
set_standard_16_palette:
    pusha
    push es

    mov ax, ds
    mov es, ax
    mov dx, palette_16_table

    ; Функция BIOS: Set Block of DAC Color Registers (ah=10h, al=12h)
    ; bx - начальный регистр, cx - кол-во регистров, es:dx - таблица RGB
    xor bx, bx
    mov cx, 16
    mov ax, 0x1012
    int 0x10

    pop es
    popa
    ret

; Стандартная 16-цветная палитра VGA/EGA, R,G,B по 6 бит (0-63) на канал -
; тот же порядок и те же значения, что подразумевает drivers::vga::Color
; в kernel32 (Black,Blue,Green,Cyan,Red,Magenta,Brown,LightGray,DarkGray,
; LightBlue,LightGreen,LightCyan,LightRed,Pink,Yellow,White)
palette_16_table:
    db 0,0,0,        0,0,42,      0,42,0,       0,42,42
    db 42,0,0,       42,0,42,     42,21,0,      42,42,42
    db 21,21,21,     21,21,63,    21,63,21,     21,63,63
    db 63,21,21,     63,21,63,    63,63,21,     63,63,63


; > Включение текстового режима (80x25)
enable_vga_textmode:
    push ax

    ; Функция BIOS: Установка режима (ah = 0, al = 03h)
    mov ax, TEXT_MODE_80x25
    int 0x10

    pop ax
    ret


; > Получение информации о видеорежиме
; Вывод:
;  - al: номер видеорежима
;  - ah: кол-во стобцов
;  - bh: номер страницы
get_vga_mode:
    mov ah, 0x0F
    int 0x10

    ret


; > Очистка экрана выбранным цветом
; Параметры:
;  - al: цвет очистки (0 - черный)
vga_fill:
    push cx
    push es
    push di

    ; Настройка видеосегмента (es:di)
    mov cx, VGA_SEGMENT
    mov es, cx
    xor di, di

    ; Заполнение видеопамяти es:di через al, содержащий цвет
    mov cx, VGA_WIDTH * VGA_HEIGHT
    rep stosb

    pop di
    pop es
    pop cx
    ret


; > Отрисовка одного пикселя
; Параметры:
;  - bx, ax: X, Y (0-319, 0-199)
;  - dl: цвет (0-255)
vga_set_pixel:
    push ax
    push cx
    push di
    push es

    ; Проверка на выход за границы экрана
    cmp bx, VGA_WIDTH
    jae .done           ; Если X >= 320, выходим
    cmp ax, VGA_HEIGHT
    jae .done           ; Если Y >= 200, выходим

    ; Вычисление адреса пикселя
    ; > di = Y * 320 + X (ax - Y, bx - X)
    push dx            ; *Сохраняем цвет пикселя
    mov cx, VGA_WIDTH
    mul cx
    add ax, bx
    mov di, ax

    ; Запись в видеопамять (es:di)
    mov cx, VGA_SEGMENT
    mov es, cx
    pop dx           ; *Восстанавливаем цвет пикселя
    mov [es:di], dl

.done:
    pop es
    pop di
    pop cx
    pop ax
    ret


; > Получение данных одного пикселя
; Параметры:
;  - bx, ax: X, Y (0-319, 0-199)
; Вывод:
;  - dl: цвет найденного пикселя
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка)
vga_get_pixel:
    push ax
    push cx
    push di
    push es

    ; Проверка на выход за границы экрана
    cmp bx, VGA_WIDTH
    jae .fail           ; Если X >= 320, выходим
    cmp ax, VGA_HEIGHT
    jae .fail           ; Если Y >= 200, выходим

    ; Вычисляем адрес искомого пикселя
    ; > di = Y * 320 + X (ax - Y, bx - X)
    mov cx, VGA_WIDTH
    mul cx
    add ax, bx
    mov di, ax

    ; Чтение из видеопамяти (es:di)
    mov cx, VGA_SEGMENT
    mov es, cx
    mov dl, [es:di]

    clc
    jmp .done

; Выход за границы экрана
.fail:
    xor dl, dl
    stc

.done:
    pop es
    pop di
    pop cx
    pop ax
    ret


; > Отрисовка горизонтальной линии
; Параметры:
;  - bx, cx: X1, X2 (Начало:Конец)
;  - ax: Y
;  - dl: цвет (0-255)
vga_draw_hline:
    push bx

.loop:
    ; Рисуем пиксель линии
    call vga_set_pixel

    ; Сдвигаемся вправо по X
    inc bx
    cmp bx, cx
    jle .loop

.done:
    pop bx
    ret


; > Отрисовка вертикальной линии
; Параметры:
;  - bx: X
;  - ax, cx: Y1, Y2 (Начало:Конец)
;  - dl: цвет (0-255)
vga_draw_vline:
    push ax

.loop:
    ; Рисуем пиксель линии
    call vga_set_pixel

    ; Сдвигаемся вниз по Y
    inc ax
    cmp ax, cx
    jle .loop

.done:
    pop ax
    ret


; > Отрисовка залитого прямоугольника
; Параметры:
;  - bx: X1, ax: Y1 (Левый верхний угол)
;  - cx: X2, dx: Y2 (Правый нижний угол)
;  - si: цвет заливки (0-255)
vga_draw_rect:
    push ax

.loop:
    ; Рисуем горизонтальную строку прямоугольника
    ; (dx - сохраняем цвет пикселя)
    push dx
    mov dx, si
    call vga_draw_hline

    ; Переход на след. строку прямоугольника
    inc ax
    pop dx
    cmp ax, dx
    jle .loop

.done:
    pop ax
    ret
