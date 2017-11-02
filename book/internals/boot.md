# Boot process

## Sequence diagram

```sequence
Title: Boot process
BIOS->CDROM: Read 1st\nsector and\nrun it
CDROM->GRUB: Read rest of\nthe image and\nrun GRUB Kernel
GRUB-->>BIOS: Fetch system information
Note over GRUB: Enter protected mode
GRUB-->>CDROM: Load grub.cfg
Note over GRUB: Show boot screen
GRUB-->>Kernel: Load kernel.bin
Note over GRUB: Fill Multiboot data
GRUB->Kernel: Call krnl_start32
Note over Kernel: Enter long mode
Note over Kernel: Call krnl_start64
Note over Kernel: Call krnl_main
```

*(this diagram may not resemble the real state of art)*

## Bootable image structure 

The bootable ISO image of the kernel is created using `grub-mkrescure` program. This utility creates ISO image, whose first sector contains code which performs necessary system initialization and calls GRUB code located further in the image.

GRUB is built with `iso9660` and `biosdisk` modules, which enable it to read ISO 9660 filesystems and communicate with BIOS. It performs further initialization (including entering protected mode), reads `grub.cfg` and displays boot selection screen.
