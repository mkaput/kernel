/* Set kernel entry point */
ENTRY(krnl_start32)

SECTIONS {
    /* Set load address of first section to 1MiB,
       this is conventional place to load a kernel. */
    . = 1M;

    /* Ensure that the multiboot header is at the beginning. */
    .boot : {
        KEEP(*(.multiboot))
    }

    .text : {
        *(.text)
    }

	/* Read-write data (uninitialized) and stack */
	.bss : {
		*(COMMON)
		*(.bss)
	}

    /* Read-only data */
    .rodata : {
        *(.rodata)
    }
}
