;; sysenter.asm, sysenter for Realix
;; Copyright 2026 Alexander Silaev
;; built on nasm, for x86

bits 32

section .text

global _sysenter_init
global sysenter_handler
extern sysenter_dispatch
extern set_tss_esp0

; прастити, но мне впадлу это переводить
; void sysenter_init(u32 kernel_esp0). Sets up IA32_SYSENTER_CS/ESP/EIP MSRs
; AI AI AI AI AI AI AI AI AI, Microslop AI, Googleslop AI, Appleslop AI, all AI, and Xbox, Watch TV 
_sysenter_init:
    push ebp
    mov ebp, esp

    ; SS = CS + 8 = 0x10
    mov ecx, 0x174
    mov eax, 0x08
    xor edx, edx
    wrmsr

    ; IA32_SYSENTER_ESP = kernel stack top
    mov ecx, 0x175
    mov eax, [ebp + 8]
    xor edx, edx
    wrmsr

    ; IA32_SYSENTER_EIP = handler
    mov ecx, 0x176
    mov eax, sysenter_handler
    xor edx, edx
    wrmsr

    ; set TSS esp0 too
    push dword [ebp + 8]
    call set_tss_esp0
    add esp, 4

    pop ebp
    ret

; sysenter entry point
; Hw state on entry:
;   CS = IA32_SYSENTER_CS
;   EIP = IA32_SYSENTER_EIP
;   SS = IA32_SYSENTER_CS + 8
;   ESP = IA32_SYSENTER_ESP
;
; User passed:
;   EAX = syscall number
;   EBX = arg1
;   ESI = arg2
;   EDI = arg3
;   EDX = return EIP (set by syscall stub)
;   ECX = return ESP (set by syscall stub)

sysenter_handler:
    ; At this point:
    ; - ESP = kernel stack (from MSR)
    ; - interrupts disabled? NO, sysenter doesn't touch IF
    ; - We need to save user context
    cli ; safety, offs interrupts
    ; Save return state (EDX=rip ECX=rsp) - sysexit needs these
    push edx ; user return EIP
    push ecx ; user return ESP

    ; Save callee-saved regs + args for C
    push ebp
    push esi
    push edi
    push ebx
    
    push eax ; saving EAX before call

    ; update TSS esp0 for nested interrupts
    mov eax, esp
    push eax
    call set_tss_esp0
    add esp, 4

    pop eax ; return eax

    ; call c dispatcher
    ; stack: ebx, edi, esi, ebp, ecx, edx
    ; args: arg3=edi, arg2=esi, arg1=ebx, num=eax
    push edi ; arg3
    push esi ; arg2
    push ebx ; arg1
    push eax ; num
    call sysenter_dispatch
    add esp, 16 ; clean 4 args

    ; eax = return value 

    ; Restore context
    pop ebx
    pop edi
    pop esi
    pop ebp
    
    ; Get return EIP/eSP for sysexit
    pop ecx ; user esp
    pop edx ; user eip
	.debug_loop:
		jmp .debug_loop
    ; sysexit sets:
    ;   CS = IA32_SYSENTER_CS + 16 = 0x18 (user code)
    ;   SS = IA32_SYSENTER_CS + 24 = 0x20 (user data)
    ;   EIP = EDX
    ;   ESP = ECX
    sti ; enable interrupts 
    sysexit