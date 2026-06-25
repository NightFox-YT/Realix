; © Realix > Print new line
; (14.06.26) v0.06
; ================

; > Print CR/LF on screen.
print_new_line:
    push ax
    push bx

%ifdef KERNEL_CONSOLE
    mov al, 0x0D
    call console_putc
    mov al, 0x0A
    call console_putc
%else
    mov ah, 0x0E
    xor bx, bx

    mov al, 0x0D
    int 0x10
    mov al, 0x0A
    int 0x10
%endif

    pop bx
    pop ax
    ret
