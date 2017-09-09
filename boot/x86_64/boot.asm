; Kernel entry point for x86_64 arch

global krnl_start

section .text
bits 32
krnl_start:
    ; print `OK` to screen
    mov dword [0xb8000], 0x2f4b2f4f
    hlt
