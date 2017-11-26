OUTPUT_FORMAT(elf64-x86-64)
ENTRY(krnl_start32)

SECTIONS {
    /* Set load address of first section to 1MiB,
       this is conventional place to load a kernel. */
    . = 1M;

    /* Read-only data and Multiboot header.
       We share the section for both parts in order to not waste whole page
       for small Multiboot header. Because the header has to be at the
       very beginning, this section comes first. */
    .rodata : {
        KEEP(*(.multiboot))
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .text : {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4K);
    }

    .data : {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .got : {
        *(.got)
        . = ALIGN(4K);
    }

    .got.plt : {
        *(.got.plt)
        . = ALIGN(4K);
    }

    .data.rel.ro : ALIGN(4K) {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .gcc_except_table : ALIGN(4K) {
        *(.gcc_except_table)
        . = ALIGN(4K);
    }
}
