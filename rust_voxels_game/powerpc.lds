SECTIONS
{
	. = 0;
	.head : {
		KEEP(*(.head))
 	}
	. = 0x1000;
	.text : { *(.text) }
	.data : { *(.data) }
	.rodata : { *(.rodata) }
	.bss : { *(.bss) }
	/DISCARD/ : { *(.note.gnu.build-id) }
}
