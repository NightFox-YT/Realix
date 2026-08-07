/* © Realix > 32-bit RLX C Application Linker Script */
ENTRY(_start)

SECTIONS
{
    . = 0x00400000;

    .header : {
        BYTE(0x52); /* 'R' */
        BYTE(0x4C); /* 'L' */
        BYTE(0x58); /* 'X' */
        BYTE(0x32); /* Mode 32 */
        LONG(_start - 0x00400000);
        LONG(code_end - _start);
        LONG(0);
    }

    .text : {
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        *(.bss*)
        *(COMMON)
    }

    code_end = .;
}
