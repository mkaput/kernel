/* Set kernel entry point */
ENTRY(krnl_start)

SECTIONS {
    /* Set load address of first section to 1MiB,
       this is conventional place to load a kernel. */
    . = 1M;

    /* Ensure that the multiboot header is at the beginning. */
    .boot : {
        *(.multiboot_header)
    }

    .text : {
        *(.text)
    }
}
