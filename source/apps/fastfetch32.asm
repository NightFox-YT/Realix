; © Realix > FastFetch System Information Tool for Realix OS (.RLX 32-bit)
; Runs in Protected Mode Ring 3 using System API (INT 0x80)
; ======================================================================

bits 32
org 0x00400000

%include 'shared/rlx.inc'

rlx_header:
    db RLX_MAGIC_0, RLX_MAGIC_1, RLX_MAGIC_2, RLX_MODE_32
    dd app_entry - rlx_header ; Entry offset (0x10)
    dd code_end - app_entry   ; Code size
    dd 0                      ; Reserved

app_entry:
    ; Очистка экрана
    mov eax, SYS_CLEAR
    int 0x80

    ; Печать системной информации FastFetch
    mov eax, SYS_PRINT_STRING
    mov esi, fastfetch_art
    int 0x80

    ; Завершение работы (возврат в Ring 0 Shell)
    mov eax, SYS_EXIT
    int 0x80

; --- Системная информация FastFetch ---
fastfetch_art:
    db '  _____            _ _       ', 0x0D, 0x0A
    db ' |  __ \          | (_)      ', 0x0D, 0x0A
    db ' | |__) |___  __ _| |___  __ ', 0x0D, 0x0A
    db ' |  _  // _ \/ _` | | \ \/ / ', 0x0D, 0x0A
    db ' | | \ \  __/ (_| | | |>  <  ', 0x0D, 0x0A
    db ' |_|  \_\___|\__,_|_|_/_/\_\ ', 0x0D, 0x0A
    db 0x0D, 0x0A
    db ' User@Realix-OS', 0x0D, 0x0A
    db ' --------------', 0x0D, 0x0A
    db ' OS:           Realix OS v0.1 (32-bit Protected Mode)', 0x0D, 0x0A
    db ' Kernel:       Realix Hybrid Kernel (kernel16 + kernel32)', 0x0D, 0x0A
    db ' Privilege:    Ring 3 (User Space via INT 0x80 System API)', 0x0D, 0x0A
    db ' Format:       .RLX (Realix Executable Spec v1.0)', 0x0D, 0x0A
    db ' Architecture: x86 i386', 0x0D, 0x0A
    db ' Memory:       128 MB RAM (Identity Mapped)', 0x0D, 0x0A
    db ' Shell:        Realix Ring 0 Shell', 0x0D, 0x0A
    db ' Display:      VGA Color Text Mode (80x25)', 0x0D, 0x0A
    db 0x0D, 0x0A
    db ' Palette:      [Red] [Green] [Yellow] [Blue] [Magenta] [Cyan] [White]', 0x0D, 0x0A
    db 0x0D, 0x0A, 0

code_end:
