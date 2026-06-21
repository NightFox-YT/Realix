ENTRY(_start)
OUTPUT_FORMAT("elf32-i386")

SECTIONS
{
    . = 0x20000;

    .text : {
        build/entry32.o(.text)
        *(.text)
    }

    .data : { *(.data) }
    .rodata : { *(.rodata) }
    .bss : { *(.bss) }
}