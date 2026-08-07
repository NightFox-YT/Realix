; © Realix > RLXFetch 16-bit Assembly System Information Utility (.RLX 16-bit)
; =========================================================================

bits 16
org 0x0000

%include 'shared/rlx.inc'

rlx_header:
    db RLX_MAGIC_0, RLX_MAGIC_1, RLX_MAGIC_2, RLX_MODE_16
    dw app_entry - rlx_header
    dw code_end - app_entry
    dd 0

app_entry:
    mov ax, SYS_CLEAR
    int 0x80

    mov ax, SYS_PRINT_STRING
    mov si, msg16_header
    int 0x80

    call print_blue_r_16

    mov ax, SYS_PRINT_STRING
    mov si, msg16_info
    int 0x80

    mov ax, SYS_EXIT
    int 0x80

print_blue_r_16:
    mov si, logo_r_text
.loop:
    lodsb
    test al, al
    jz .done
    cmp al, 'R'
    jne .normal
    mov ah, 0x0E
    mov bx, 0x0009 ; LightBlue
    int 0x10
    jmp .loop
.normal:
    mov ah, 0x0E
    mov bx, 0x0007 ; LightGray
    int 0x10
    jmp .loop
.done:
    ret

msg16_header: db 0x0D, 0x0A, '   === RLXFetch 16-bit System Information (Assembly) ===', 0x0D, 0x0A, 0x0D, 0x0A, 0
msg16_info:   db 0x0D, 0x0A, '   User@realix-system', 0x0D, 0x0A, '   ------------------', 0x0D, 0x0A, '   OS:          Realix OS v0.1 (16-bit Real Mode)', 0x0D, 0x0A, '   Kernel:      Realix 16-bit Kernel (kernel16)', 0x0D, 0x0A, '   Mode:        16-bit Real Mode', 0x0D, 0x0A, '   PATH:        /bin;/apps;/', 0x0D, 0x0A, '   Executable:  rlxfetch16.rlx (NASM 16-bit)', 0x0D, 0x0A, 0x0D, 0x0A, 0

logo_r_text:
    db '   RRRRRRRRRRRRRRRRR   ', 0x0D, 0x0A
    db '   RR             RR   ', 0x0D, 0x0A
    db '   RR             RR   ', 0x0D, 0x0A
    db '   RRRRRRRRRRRRRRRRR   ', 0x0D, 0x0A
    db '   RR         RR       ', 0x0D, 0x0A
    db '   RR          RR      ', 0x0D, 0x0A
    db '   RR           RR     ', 0x0D, 0x0A, 0

code_end:
