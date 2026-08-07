; © Realix > RLXFetch Universal Binary App (16-bit Real Mode & 32-bit Ring 3)
; =========================================================================

bits 16
org 0x0000

%include 'shared/rlx.inc'

rlx_header:
    db RLX_MAGIC_0, RLX_MAGIC_1, RLX_MAGIC_2, RLX_MODE_UNIVERSAL
    dw entry16 - rlx_header
    dw code16_end - entry16
    dd entry32 - rlx_header
    dd code32_end - entry32

; =========================================================================
; 16-bit Real Mode Payload
; =========================================================================
entry16:
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

msg16_header: db 0x0D, 0x0A, '   === RLXFetch Universal App (16-bit Real Mode) ===', 0x0D, 0x0A, 0x0D, 0x0A, 0
msg16_info:   db 0x0D, 0x0A, '   OS:          Realix OS v0.1 (16-bit Real Mode)', 0x0D, 0x0A, '   Kernel:      Realix 16-bit Kernel (kernel16)', 0x0D, 0x0A, '   Mode:        16-bit Real Mode', 0x0D, 0x0A, '   PATH:        /bin;/apps;/', 0x0D, 0x0A, '   Binary Spec: .RLX Universal Dual-Mode (Spec v2.0)', 0x0D, 0x0A, 0x0D, 0x0A, 0

logo_r_text:
    db '   RRRRRRRRRRRRRRRRR   ', 0x0D, 0x0A
    db '   RR             RR   ', 0x0D, 0x0A
    db '   RR             RR   ', 0x0D, 0x0A
    db '   RRRRRRRRRRRRRRRRR   ', 0x0D, 0x0A
    db '   RR         RR       ', 0x0D, 0x0A
    db '   RR          RR      ', 0x0D, 0x0A
    db '   RR           RR     ', 0x0D, 0x0A, 0

code16_end:

; =========================================================================
; 32-bit Protected Mode Ring 3 Payload
; =========================================================================
bits 32

entry32:
    mov eax, SYS_CLEAR
    int 0x80

    mov eax, SYS_PRINT_STRING
    mov esi, msg32_header
    int 0x80

    call print_blue_r_32

    ; Запрос живой системной информации через INT 0x80 SYS_GET_SYSINFO
    mov eax, SYS_GET_SYSINFO
    mov edi, sys_info_buf
    int 0x80

    mov eax, SYS_PRINT_STRING
    mov esi, msg32_info1
    int 0x80

    ; Печать CPU Vendor
    mov eax, SYS_PRINT_STRING
    mov esi, sys_info_buf + 80 ; cpu_vendor offset
    int 0x80

    mov eax, SYS_PRINT_STRING
    mov esi, msg32_info2
    int 0x80

    mov eax, SYS_EXIT
    int 0x80

print_blue_r_32:
    mov esi, logo_r_text
.loop32:
    mov al, [esi]
    inc esi
    test al, al
    jz .done32
    cmp al, 'R'
    jne .normal32

    mov eax, SYS_PUTCHAR
    mov edx, 'R'
    mov ebx, 1 ; LightBlue
    int 0x80
    jmp .loop32

.normal32:
    mov eax, SYS_PUTCHAR
    mov edx, eax
    mov dl, al
    mov ebx, 7 ; LightGray
    int 0x80
    jmp .loop32

.done32:
    ret

; Буфер системной информации (96 байт)
sys_info_buf: times 96 db 0

msg32_header: db 0x0D, 0x0A, '   === RLXFetch Universal App (32-bit Protected Mode Ring 3) ===', 0x0D, 0x0A, 0x0D, 0x0A, 0
msg32_info1:  db 0x0D, 0x0A, '   OS:          Realix OS v0.1 (32-bit Protected Mode)', 0x0D, 0x0A, '   Kernel:      Realix Hybrid Kernel (kernel32)', 0x0D, 0x0A, '   Privilege:   Ring 3 User Mode (INT 0x80 System API)', 0x0D, 0x0A, '   CPU Vendor:  ', 0
msg32_info2:  db 0x0D, 0x0A, '   RAM Memory:  128 MB RAM (Identity Mapped)', 0x0D, 0x0A, '   PATH:        /bin;/apps;/', 0x0D, 0x0A, '   Binary Spec: .RLX Universal Dual-Mode (Spec v2.0)', 0x0D, 0x0A, 0x0D, 0x0A, 0

code32_end:
