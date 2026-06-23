ENTRY(_start)
OUTPUT_FORMAT("elf32-i386")

SECTIONS
{
    . = 0x20000;

    .text : {
        build/entry32.o(.text)
        *(.text)
    }

    .rodata : { 
        *(.rodata)
        *(.rodata.*) 
    }

    .data : {
        *(.data)
        *(.data.*) 
    }

    .bss (NOLOAD) : ALIGN(4) {
        __bss_start = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        __bss_end = .;
    }

}