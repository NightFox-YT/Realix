; Realix Kernel16 console
; Keeps a small text scrollback buffer and can redraw it with PageUp/PageDown.

bits 16

CONSOLE_COLS         equ 80
CONSOLE_ROWS         equ 25
CONSOLE_BUFFER_LINES equ 100
CONSOLE_PAGE_STEP    equ 5
CONSOLE_ATTR         equ 0x07

console_init:
    push ax
    push cx
    push di

    mov di, console_buffer
    mov cx, CONSOLE_COLS * CONSOLE_BUFFER_LINES
    mov al, ' '
    rep stosb

    mov word [console_line], 0
    mov byte [console_col], 0
    mov word [console_count], 1
    mov word [console_view], 0

    pop di
    pop cx
    pop ax
    ret

; Print AL and save it to the scrollback buffer.
console_putc:
    push ax
    push bx
    push cx
    push dx
    push si
    push di

    mov [console_char], al

    cmp al, 0x0D
    je .carriage_return

    cmp al, 0x0A
    je .line_feed

    cmp byte [console_col], CONSOLE_COLS
    jb .store_char
    call console_newline_internal

.store_char:
    mov ax, [console_line]
    mov bx, CONSOLE_COLS
    mul bx
    mov di, console_buffer
    add di, ax

    xor ax, ax
    mov al, [console_col]
    add di, ax

    mov al, [console_char]
    mov [di], al

    cmp word [console_view], 0
    jne .after_display

    mov ah, 0x0E
    xor bx, bx
    int 0x10

.after_display:
    inc byte [console_col]
    cmp byte [console_col], CONSOLE_COLS
    jb .done
    call console_newline_internal
    jmp .done

.carriage_return:
    cmp word [console_view], 0
    jne .done
    mov ah, 0x0E
    xor bx, bx
    int 0x10
    jmp .done

.line_feed:
    call console_newline_internal
    cmp word [console_view], 0
    jne .done
    mov al, 0x0A
    mov ah, 0x0E
    xor bx, bx
    int 0x10

.done:
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

console_newline_internal:
    push ax

    mov byte [console_col], 0

    mov ax, [console_line]
    inc ax
    cmp ax, CONSOLE_BUFFER_LINES
    jb .line_ok
    xor ax, ax

.line_ok:
    mov [console_line], ax

    mov ax, [console_count]
    cmp ax, CONSOLE_BUFFER_LINES
    jae .clear_line
    inc ax
    mov [console_count], ax

.clear_line:
    call console_clear_current_line

    pop ax
    ret

console_clear_current_line:
    push ax
    push bx
    push cx
    push di

    mov ax, [console_line]
    mov bx, CONSOLE_COLS
    mul bx
    mov di, console_buffer
    add di, ax

    mov cx, CONSOLE_COLS
    mov al, ' '
    rep stosb

    pop di
    pop cx
    pop bx
    pop ax
    ret

; PageUp: show older lines.
console_page_up:
    push ax

    mov ax, [console_count]
    cmp ax, 1
    jbe .done
    dec ax                         ; max view offset

    mov [console_max_view], ax
    mov ax, [console_view]
    add ax, CONSOLE_PAGE_STEP
    cmp ax, [console_max_view]
    jbe .store
    mov ax, [console_max_view]

.store:
    mov [console_view], ax
    call console_redraw

.done:
    pop ax
    ret

; PageDown: show newer lines.
console_page_down:
    push ax

    mov ax, [console_view]
    cmp ax, CONSOLE_PAGE_STEP
    ja .sub_step
    xor ax, ax
    jmp .store

.sub_step:
    sub ax, CONSOLE_PAGE_STEP

.store:
    mov [console_view], ax
    call console_redraw

    pop ax
    ret

; Return to live bottom view.
console_to_bottom:
    cmp word [console_view], 0
    je .done
    mov word [console_view], 0
    call console_redraw
.done:
    ret

console_redraw:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    mov ax, [console_line]
    sub ax, [console_view]
    call console_wrap_ax
    sub ax, CONSOLE_ROWS - 1
    call console_wrap_ax
    mov [console_draw_line], ax

    mov ax, 0xB800
    mov es, ax
    xor di, di
    mov dx, CONSOLE_ROWS

.row_loop:
    mov ax, [console_draw_line]
    mov bx, CONSOLE_COLS
    mul bx
    mov si, console_buffer
    add si, ax

    mov cx, CONSOLE_COLS

.col_loop:
    lodsb
    mov ah, CONSOLE_ATTR
    stosw
    loop .col_loop

    mov ax, [console_draw_line]
    inc ax
    call console_wrap_ax
    mov [console_draw_line], ax

    dec dx
    jnz .row_loop

    mov ah, 0x02
    xor bh, bh
    mov dh, CONSOLE_ROWS - 1
    mov dl, 0
    cmp word [console_view], 0
    jne .set_cursor
    mov dl, [console_col]
    cmp dl, CONSOLE_COLS
    jb .set_cursor
    mov dl, CONSOLE_COLS - 1

.set_cursor:
    int 0x10

    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Wrap AX to 0..CONSOLE_BUFFER_LINES-1.
console_wrap_ax:
.wrap:
    cmp ax, CONSOLE_BUFFER_LINES
    jb .done
    cmp ax, 0x8000
    jae .negative
    sub ax, CONSOLE_BUFFER_LINES
    jmp .wrap

.negative:
    add ax, CONSOLE_BUFFER_LINES
    jmp .wrap

.done:
    ret

console_line:      dw 0
console_count:     dw 1
console_view:      dw 0
console_max_view:  dw 0
console_draw_line: dw 0
console_col:       db 0
console_char:      db 0

console_buffer: times CONSOLE_COLS * CONSOLE_BUFFER_LINES db ' '
