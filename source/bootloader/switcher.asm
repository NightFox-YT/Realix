; © Realix > CPU Mode Switcher
; Исправленная версия
; ===================

%ifndef BOOT_SWITCHER_ASM
%define BOOT_SWITCHER_ASM

bits 16

%include 'shared/config.asm'


; ============================================================
; ВЫБОР РЕЖИМА
; ============================================================

boot_switcher:
    cld

    mov si, str_choose_mode
    call print

.wait_key:
    xor ah, ah
    int 0x16

    cmp al, '1'
    je .load_kernel16

    cmp al, '2'
    je .load_kernel32

    jmp .wait_key


; ============================================================
; KERNEL16
; ============================================================

.load_kernel16:
    call print_new_line

    mov si, msg_loading_16
    call print

    mov si, kernel16_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    mov dl, [boot_drive_num]
    call file_load
    jc .file_error

    mov ax, KERNEL_LOAD_SEGMENT
    mov ds, ax
    mov es, ax

    ; Kernel16 самостоятельно читает PCINFO по фиксированному адресу.
    mov di, PCINFO_ADDR

    cld
    jmp KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET


; ============================================================
; KERNEL32
; ============================================================

.load_kernel32:
    call print_new_line

    mov si, msg_loading_32
    call print

    mov si, kernel32_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    mov dl, [boot_drive_num]
    call file_load
    jc .file_error

    ; Физический адрес временной GDT.
    xor eax, eax
    mov ax, ds
    shl eax, 4
    add eax, gdt_start
    mov [gdt_descriptor + 2], eax

    ; Физический адрес точки входа Protected Mode.
    xor eax, eax
    mov ax, ds
    shl eax, 4
    add eax, pmode_entry
    mov [pmode_target_offset], eax

    cli
    cld

    ; Fast A20 gate.
    in al, 0x92
    and al, 0xFE
    or al, 0x02
    out 0x92, al

    lgdt [gdt_descriptor]

    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp dword far [pmode_target]


; ============================================================
; ОШИБКА ЗАГРУЗКИ
; ============================================================

.file_error:
    ; AL содержит код FAT12_ERROR_*.
    mov si, msg_kernel_load_error
    jmp error_handler


; ============================================================
; FAR POINTER PROTECTED MODE
; ============================================================

align 4

pmode_target:
pmode_target_offset:
    dd 0

pmode_target_selector:
    dw 0x08


; ============================================================
; ВРЕМЕННАЯ GDT
; ============================================================

align 8

gdt_start:
    ; Null descriptor.
    dq 0

gdt_code:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10011010b
    db 11001111b
    db 0x00

gdt_data:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10010010b
    db 11001111b
    db 0x00

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd 0


; ============================================================
; PROTECTED MODE ENTRY
; ============================================================

bits 32

pmode_entry:
    cld

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov esp, 0x90000
    mov ebp, esp

    ; EBX — адрес PCINFO для точки входа Rust.
    mov ebx, PCINFO_ADDR

    mov eax, KERNEL32_PHYS_ADDR
    jmp eax

.halt:
    cli
    hlt
    jmp .halt


; ============================================================
; ДАННЫЕ REAL MODE
; ============================================================

bits 16

str_choose_mode:
    db '[+] Select OS Mode:', ENTER
    db '  [1] 16-bit Real Mode', ENTER
    db '  [2] 32-bit Protected Mode (Rust)', ENTER, 0

msg_loading_16:
    db '[+] Loading 16-bit kernel.', ENTER, 0

msg_loading_32:
    db '[+] Loading 32-bit kernel.', ENTER, 0

msg_kernel_load_error:
    db '[!] Failed to load kernel file.', 0

kernel16_filename:
    db 'KERNEL16BIN'

kernel32_filename:
    db 'KERNEL32BIN'

%endif
