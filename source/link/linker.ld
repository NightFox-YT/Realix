ENTRY(_start)
OUTPUT_FORMAT("binary")

SECTIONS
{
	. = 0x20000;

	.text : {
		entry32.o(.text)
		*(.text)
	}

	.data : { *(.data) }
	.rodata : { *(.rodata) }
	.bss : { *(.bss) }
}