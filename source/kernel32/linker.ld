ENTRY(_start)

SECTIONS
{
    /* Физический адрес загрузки Kernel32. */
    . = 0x10000;

    .text : ALIGN(4)
    {
        /* Точка входа всегда должна находиться первой. */
        KEEP(*(.text.entry))

        *(.text)
        *(.text.*)
    }

    .rodata : ALIGN(4)
    {
        *(.rodata)
        *(.rodata.*)
    }

    .data : ALIGN(4)
    {
        *(.data)
        *(.data.*)
    }

    /*
     * Секция BSS отсутствует в плоском binary-файле, поэтому ядро
     * самостоятельно обнуляет диапазон __bss_start..__bss_end.
     */
    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        __bss_end = .;
    }

    __kernel_end = .;

    /* Стек ядра: 0x80000..0x90000. Все секции ядра должны находиться ниже. */
    ASSERT(__kernel_end <= 0x80000, "kernel overlaps the reserved kernel stack")

    /* Эти секции объектных файлов мы выкидываем из бинарника */
    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note)
        *(.note.*)
        *(.debug)
        *(.debug.*)
    }
}
