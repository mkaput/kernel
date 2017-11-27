# Booting the kernel

The kernel is distributed as bootable image that can be used as LiveCD or LiveUSB, which eases its educational usage. The booting process is highly dependent on Multiboot compatible bootloader, usually GRUB 2.

```sequence
Title: Entering krnl_main

participant BIOS\nUEFI as BIOS
participant CDROM
participant GRUB
participant Kernel\n(ASM) as ASM
participant Kernel\n(Rust) as Rust

BIOS-->CDROM: Load 1st sector
CDROM-->BIOS:
BIOS->CDROM: Jump to\n1st sector
CDROM-->GRUB: Read rest of\nthe image
GRUB-->CDROM:
CDROM->GRUB: Jump to\nGRUB Kernel
GRUB-->BIOS: Fetch system information
BIOS-->GRUB:
Note over GRUB: Enter protected mode
GRUB-->>CDROM: Load grub.cfg
CDROM-->GRUB:
Note over GRUB: Show boot screen
GRUB-->ASM: Load kernel.bin
ASM-->GRUB:
Note over GRUB: Fill Multiboot\ninfo table
GRUB->ASM: Jump to\nkrnl_start32
Note over ASM: Perform basic system checks,\nprepare min. environment for\nRust code, enter long mode\nand far jump to krnl_start64
ASM->Rust: Call krnl_main
```

Very nice and more detailed description on how do early stages of GRUB 2 work is in the [Linux Inside] book, chapter [From bootloader to kernel][li-boot]. Note that Linux has its own [boot protocol][linux-boot] which differs from Multiboot.

## Bootable image structure 

The bootable ISO image of the kernel is created using `grub-mkrescure` program. This utility creates ISO image, whose first sector contains code which performs necessary system initialization and calls GRUB code located further in the image.

GRUB is built with `iso9660` and `biosdisk` modules, which enable it to read ISO 9660 filesystems and communicate with BIOS. It performs further initialization (including entering protected mode), reads `grub.cfg` and displays boot selection screen.

[Linux Inside]: https://www.gitbook.com/book/0xax/linux-insides/details
[li-boot]: https://0xax.gitbooks.io/linux-insides/content/Booting/linux-bootstrap-1.html#bootloader
[linux-boot]: https://github.com/torvalds/linux/blob/16f73eb02d7e1765ccab3d2018e0bd98eb93d973/Documentation/x86/boot.txt
