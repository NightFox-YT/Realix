# © Realix > 32-bit RLX Entry Point Assembly for C Applications
# =============================================================

.code32
.global _start

.section .header, "ax"
.byte 'R', 'L', 'X', 0x32
.long _start - rlx_header_start
.long 0x1000
.long 0

rlx_header_start:
.section .text
_start:
    call main
    mov $3, %eax
    int $0x80
    hlt
