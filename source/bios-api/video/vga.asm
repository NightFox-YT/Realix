; © Realix > VGA driver
; (13.06.26) v0.06
; ================

; Основные константы
%include 'shared/config.asm'

; Константы экрана VGA
VGA_WIDTH   equ 320
VGA_HEIGHT  equ 200
VGA_SEGMENT equ 0xA000

; > Инициализация видеорежима 13h (320x200, 256 цветов)
vga_video_mode:
    push ax

    ; Установка режима (ah = 0, al = 13h)
    mov ax, 0x0013
    int 0x10

    pop ax
    ret

; > Включение текстового режима (80x25)
vga_text_mode:
    push ax

    ; Установка режима (ah = 0, al = 03h)
    mov ax, 0x0003
    int 0x10

    pop ax
    ret

; > Очистка экрана выбранным цветом
; Параметры:
;  - al: цвет очистки (0 - черный)
vga_clear:
    push cx
    push es
    push di

    ; Настройка видеосегмента
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

    ; Вычисление адреса пикселя: di = Y * 320 + X
    push dx
    mov cx, VGA_WIDTH
    mul cx
    add ax, bx
    mov di, ax

    ; Запись в видеопамять
    mov cx, VGA_SEGMENT
    mov es, cx
    pop dx
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
;  - CF (Carry Flag): 0 - успех, 1 - ошибка
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

    ; Вычисляем адрес искомого пикселя: di = Y * 320 + X
    mov cx, VGA_WIDTH
    mul cx
    add ax, bx
    mov di, ax

    ; Чтение из видеопамяти
    mov cx, VGA_SEGMENT
    mov es, cx
    mov dl, [es:di]
    mov [.pixel], dl

    clc
    jmp .done

; Выход за границы экрана
.fail:
    ; Сброс dl и установка CF (Carry Flag)
    stc
    xor dl, dl

.done:
    pop es
    pop di
    pop cx
    pop ax
    ret

.pixel: db 0


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
    ; (dx сохраняем, т.к. используем как цвет)
    push dx
    mov dx, si
    call vga_draw_hline

    ; Переход на след строку
    inc ax
    pop dx
    cmp ax, dx
    jle .loop

.done:
    pop ax
    ret