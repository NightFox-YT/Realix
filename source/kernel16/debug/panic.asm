; Realix Kernel16 debug helpers
; Basic real-mode panic screen and simple CPU exception handlers.

bits 16

; Install a few useful real-mode exception handlers.
; This helps when the kernel crashes after it has already started.
install_exception_handlers:
    push ax
    push es

    xor ax, ax
    mov es, ax
    mov ax, cs

    mov word [es:0x00 * 4], isr_divide_error
    mov word [es:0x00 * 4 + 2], ax

    mov word [es:0x03 * 4], isr_breakpoint
    mov word [es:0x03 * 4 + 2], ax

    mov word [es:0x04 * 4], isr_overflow
    mov word [es:0x04 * 4 + 2], ax

    mov word [es:0x05 * 4], isr_bounds
    mov word [es:0x05 * 4 + 2], ax

    mov word [es:0x06 * 4], isr_invalid_opcode
    mov word [es:0x06 * 4 + 2], ax

    mov word [es:0x0D * 4], isr_general_protection
    mov word [es:0x0D * 4 + 2], ax

    pop es
    pop ax
    ret

; Manual panic for testing from shell command.
debug_panic_manual:
    mov si, panic_manual_msg
    jmp exception_common

; Manual divide-by-zero crash for testing the installed handler.
debug_test_divzero:
    xor ax, ax
    div al
    ret

isr_divide_error:
    mov si, panic_divide_msg
    jmp exception_common

isr_breakpoint:
    mov si, panic_breakpoint_msg
    jmp exception_common

isr_overflow:
    mov si, panic_overflow_msg
    jmp exception_common

isr_bounds:
    mov si, panic_bounds_msg
    jmp exception_common

isr_invalid_opcode:
    mov si, panic_invalid_opcode_msg
    jmp exception_common

isr_general_protection:
    mov si, panic_gp_msg
    jmp exception_common

; Common panic screen.
; If called as a CPU exception, stack contains IP, CS, FLAGS.
; If called manually, those values are not meaningful, but the screen still helps.
exception_common:
    cli

    push bp
    mov bp, sp

    push ax
    push bx
    push si
    push ds

    push cs
    pop ds

    mov [panic_message_ptr], si

    ; Make sure we are in a visible text mode.
    mov ax, 0x0003
    int 0x10

    mov si, panic_title
    call print
    call print_new_line

    mov si, panic_reason
    call print
    mov si, [panic_message_ptr]
    call print
    call print_new_line

    mov si, panic_ip
    call print
    mov ax, [ss:bp + 2]
    call debug_print_hex16

    mov si, panic_cs
    call print
    mov ax, [ss:bp + 4]
    call debug_print_hex16

    mov si, panic_flags
    call print
    mov ax, [ss:bp + 6]
    call debug_print_hex16
    call print_new_line

    mov si, panic_hint_1
    call print
    call print_new_line
    mov si, panic_hint_2
    call print

.halt:
    hlt
    jmp .halt

; Print AX as four uppercase hexadecimal digits.
debug_print_hex16:
    push ax
    push bx
    push cx

    mov bx, ax
    mov cx, 4

.next_digit:
    rol bx, 4
    mov al, bl
    and al, 0x0F

    cmp al, 9
    jbe .digit
    add al, 'A' - 10
    jmp .print

.digit:
    add al, '0'

.print:
    push ax
    push bx
    mov ah, 0x0E
    xor bx, bx
    int 0x10
    pop bx
    pop ax

    loop .next_digit

    pop cx
    pop bx
    pop ax
    ret

panic_message_ptr: dw 0

panic_title:              db '*** REALIX KERNEL PANIC ***', 0
panic_reason:             db 'Reason: ', 0
panic_ip:                 db 'IP=0x', 0
panic_cs:                 db ' CS=0x', 0
panic_flags:              db ' FLAGS=0x', 0
panic_hint_1:             db 'CPU halted to keep the error visible.', 0
panic_hint_2:             db 'If this happened in QEMU, also check qemu-debug.log.', 0
panic_manual_msg:         db 'Manual panic requested', 0
panic_divide_msg:         db 'Divide error', 0
panic_breakpoint_msg:     db 'Breakpoint', 0
panic_overflow_msg:       db 'Overflow', 0
panic_bounds_msg:         db 'BOUND range exceeded', 0
panic_invalid_opcode_msg: db 'Invalid opcode', 0
panic_gp_msg:             db 'General protection fault', 0
