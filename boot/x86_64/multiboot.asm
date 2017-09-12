; Multiboot2 header
;
; Multiboot docs:
; https://www.gnu.org/software/grub/manual/multiboot2/
; http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot2.h?h=multiboot2

section .multiboot
header_start:
    %define MB_MAGIC       0xE85250D6
    %define MB_ARCH        0
    %define MB_HEADER_LEN  (header_end - header_start)

    ; The field 'magic' is the magic number identifying the header,
    ; which must be the hexadecimal value 0xE85250D6.
    dd MB_MAGIC

    ; The field 'architecture' specifies the Central Processing Unit
    ; Instruction Set Architecture. Since 'magic' isn't a palindrome it
    ; already specifies the endian-ness ISAs differing only in endianness
    ; recieve the same ID. '0' means 32-bit (protected) mode of i386.
    ; '4' means 32-bit MIPS.
    dd MB_ARCH

    ; The field 'header_length' specifies the length of Multiboot2 header in
    ; bytes including magic fields.
    dd MB_HEADER_LEN

    ; The field 'checksum' is a 32-bit unsigned value which, when added to the
    ; other magic fields (i.e. 'magic', 'architecture' and 'header_length'),
    ; must have a32-bit unsigned sum of zero.
    dd 0x100000000 - (MB_MAGIC + MB_ARCH + MB_HEADER_LEN)

    ; Required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:
