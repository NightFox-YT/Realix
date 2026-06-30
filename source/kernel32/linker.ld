ENTRY(_start)

SECTIONS
{
    . = 0x10000;

    .text : ALIGN(4)
    {
        /* Расставляем код с точкой входа в начале */
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : { *(.rodata .rodata.*) }

    .data : { *(.data .data.*) }

    .bss : { *(.bss .bss.*) }

    /* Эти секции объектных файлов мы выкидываем из бинарника */
    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame)
        *(.note .note.*)
    }
}