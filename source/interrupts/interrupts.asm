; interrupts.asm
; © 2026 Alexander Silaev
; Interrupts for Realix.
; built with nasm.

section .text
global irq0_handler
global irq1_handler

extern pit_irq_handler
extern realix_irq1_generic_handler

; макрос для сохранения контекста
%macro SAVE_CONTEXT 0
	pusha
	push ds
	push es
	push fs
	push gs
	
	mov ax, 0x10
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs,	ax
%endmacro

; макрос для восстановления регистров
%macro RESTORE_REGS 0 
    pop gs
    pop fs
    pop es
    pop ds
    popa ; гыгы попа смишно
%endmacro

; IRQ0 - Timer
irq0_handler:
    SAVE_CONTEXT
    cld

    ; pass the current ESP as argument to the C func.
    push esp
    call pit_irq_handler
    
    add esp, 4

    mov esp, eax
    
    RESTORE_REGS ;; ВНИМАНИЕ: возможно восстановление регистров уже из другого потока!
    iretd

; IRQ1 - Keyboard
irq1_handler:
    SAVE_CONTEXT
    cld
    call realix_irq1_generic_handler
    mov al, 0x20
    out 0x20, al
    RESTORE_REGS
    iretd
