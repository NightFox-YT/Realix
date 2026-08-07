; © Realix > Kernel16: System API (INT 0x80) Handler
; ====================================================

bits 16

%include 'shared/rlx.inc'

; Флаг и контекст завершения приложения
app16_running: db 0

; Инициализация вектора INT 0x80 в IVT
syscall16_init:
    push es
    xor ax, ax
    mov es, ax
    mov word [es:0x80 * 4], syscall16_entry
    mov word [es:0x80 * 4 + 2], cs
    pop es
    ret

; Точка входа для INT 0x80
syscall16_entry:
    pusha

    cmp ax, SYS_PRINT_STRING
    je .sys_print

    cmp ax, SYS_PUTCHAR
    je .sys_putchar

    cmp ax, SYS_EXIT
    je .sys_exit

    cmp ax, SYS_READ_KEY
    je .sys_read_key

    cmp ax, SYS_CLEAR
    je .sys_clear

    jmp .done

.sys_print:
    call print
    jmp .done

.sys_putchar:
    mov ah, 0x0E
    mov al, dl
    int 0x10
    jmp .done

.sys_read_key:
    mov ah, 0
    int 0x16
    mov bx, sp
    mov [ss:bx + 14], ax
    jmp .done

.sys_clear:
    call cmd_cls
    jmp .done

.sys_exit:
    mov byte [cs:app16_running], 0
    popa
    iret

.done:
    popa
    iret
