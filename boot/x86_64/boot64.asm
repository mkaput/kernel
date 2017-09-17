; Kernel entry point for x86_64 arch, 64-bit initialization code
;
; Some code is based on snippets from these pages:
; http://wiki.osdev.org/Setting_Up_Long_Mode
; https://os.phil-opp.com/

global krnl_start64

section .text
bits 64
krnl_start64:
    ; load 0 into all data segment registers
    xor ax, ax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; print `OKAY` to screen
    mov rax, 0x2f592f412f4b2f4f
    mov qword [0xb8000], rax
    hlt
