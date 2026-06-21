;; kernel.asm
; the main entry of kernel32
; (C) 2026 Alexander Silaev
; The Realix

bits 32

global _start
extern kernel_exec

section .bss
align 16
kernel_stack_bottom:
	resb 16384
kernel_stack_top

section .text
_start:
	mov ax, 0x10
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax
	mov ss, ax
	mov esp, kernel_stack_top
	mov ebp, esp

	call kernel_exec

global _jump_to_userspace
_jump_to_userspace:
    mov ecx, [esp + 4] ; Пользовательский EIP (temporary_user_stub)
    mov edx, [esp + 8] ; Пользовательский ESP (user_stack_top)

    cli 

    push 0x23          ; SS (User Data)
    push edx           ; ESP (User Stack)
    
    ; Подготовка EFLAGS
    pushf
    pop eax
    or eax, 0x200      ; Включаем прерывания (IF = 1)
    push eax
    
    push 0x1B          ; CS (User Code)
    push ecx           ; EIP (Точка входа)
    
    mov ax, 0x23
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    iretd

.halt:
	cli
	hlt
	jmp .halt