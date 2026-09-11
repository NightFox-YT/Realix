; © Realix > Command: VGA demo
; (28.07.26) v0.11
; ================
; ❗️ Зависимости: bios-api/video/vga.asm, kernel16-nightly/io/print.asm

; Цвета палитры VGA 13h, используемые в демо
VGA_DEMO_BG    equ 0x01  ; Синий фон
VGA_DEMO_RECT1 equ 0x04  ; Красный прямоугольник
VGA_DEMO_RECT2 equ 0x0E  ; Жёлтый прямоугольник
VGA_DEMO_RECT3 equ 0x0F  ; Белый прямоугольник

; > Команда демонстрации графического режима
cmd_vga:
    push ax
    push bx
    push cx
    push dx
    push si

    ; Предупреждение до смены режима
    mov si, msg_vga
    call print

    ; Ожидание любой клавиши перед стартом демо
    xor ah, ah
    int 0x16

    ; Переход в режим 320x200 (256 цветов) и заливка фона
    call vga_enable_video_mode
    mov al, VGA_DEMO_BG
    call vga_fill

    ; Три вложенных прямоугольника
    mov bx, 40
    mov ax, 30
    mov cx, 279
    mov dx, 169
    mov si, VGA_DEMO_RECT1
    call vga_draw_rect

    mov bx, 80
    mov ax, 60
    mov cx, 239
    mov dx, 139
    mov si, VGA_DEMO_RECT2
    call vga_draw_rect

    mov bx, 130
    mov ax, 85
    mov cx, 189
    mov dx, 114
    mov si, VGA_DEMO_RECT3
    call vga_draw_rect

    ; Выход по любой клавише
    xor ah, ah
    int 0x16

    ; Возврат в текстовый режим 80x25 (Заодно очищает экран)
    call vga_enable_text_mode

.done:
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Строки
msg_vga:
    db '[+] Starting VGA demo 320x200...', ENTER
    db 'Press any key to start... (After that you can press any key to exit)', 0
