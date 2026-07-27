; © Realix > Switcher CPU modes
; (21.06.26) v0.07
; ================
; ❗️ Не standalone: Подключается из initrix.asm (%include)
; ❗️ Зависимости: kernel16/io: print & print_new_line; bios-api/fat12/file_load

bits 16
%include 'shared/config.asm'

boot_switcher:
    mov si, str_choose_mode
    call print

    ; ---- Таймаут 5 секунд ----
    mov ah, 0
    int 1Ah
    add dx, 91
    adc cx, 0
    mov [timeout_ticks], cx
    mov [timeout_ticks+2], dx

.wait_key:
    mov ah, 0
    int 1Ah
    cmp cx, [timeout_ticks]
    jb .check_key
    ja .auto_32
    cmp dx, [timeout_ticks+2]
    jb .check_key
.auto_32:
    mov si, msg_auto_32
    call print
    jmp .load_kernel32

.check_key:
    mov ah, 1
    int 16h
    jz .wait_key
    mov ah, 0
    int 16h
    cmp al, '1'
    je .load_kernel16
    cmp al, '2'
    je .load_kernel32
    jmp .wait_key

.load_kernel16:
    call print_new_line
    mov si, msg_loading_16
    call print
    mov si, kernel16_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    xor di, di
    call file_load
    jc error_handler
    mov dl, [disk_current_drive]
    mov di, PCINFO_ADDR
    mov ax, KERNEL_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    jmp KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET

.load_kernel32:
    call print_new_line
    mov si, msg_loading_32
    call print
    mov si, kernel32_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    xor di, di
    call file_load
    jc error_handler

    xor eax, eax
    mov ax, ds
    shl eax, 4
    add eax, gdt_start
    mov [gdt_descriptor + 2], eax

    xor eax, eax
    mov ax, ds
    shl eax, 4
    add eax, pmode_entry
    mov [pmode_target_offset], eax

    cli
    in al, 0x92
    and al, 0xFE
    or al, 2
    out 0x92, al
    lgdt [gdt_descriptor]
    mov eax, cr0
    or eax, 0x00000001
    mov cr0, eax
    jmp dword far [pmode_target]

pmode_target:
    pmode_target_offset: dd 0
    pmode_target_sel:    dw 0x08

align 4
gdt_start:
    dd 0x0, 0x0
gdt_code:
    dw 0xFFFF, 0x0000, 0x00, 10011010b, 11001111b, 0x00
gdt_data:
    dw 0xFFFF, 0x0000, 0x00, 10010010b, 11001111b, 0x00
gdt_end:
gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

bits 32
pmode_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov ebp, 0x90000
    mov esp, ebp
    mov ebx, PCINFO_ADDR
    mov eax, KERNEL32_PHYS_ADDR
    jmp eax
    cli
    hlt
    jmp $

bits 16
str_choose_mode:
    db '[+] Select OS Mode:', ENTER
    db '  [1] 16-bit Real Mode (NASM)', ENTER
    db '  [2] 32-bit Protected Mode (Rust)', ENTER, 0

msg_auto_32:     db '[+] Auto-booting 32-bit kernel.', ENTER, 0
msg_loading_16:  db '[+] Loading 16-bit kernel.', ENTER, 0
msg_loading_32:  db '[+] Entering 32-bit Protected Mode.', ENTER, 0

kernel16_filename: db 'KERNEL16BIN'
kernel32_filename: db 'KERNEL32BIN'
timeout_ticks:     dd 0