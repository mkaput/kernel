; Kernel entry point for x86_64 arch, 64-bit initialization code
;
; THERE ARE NO CALLING CONVENTIONS IN THIS FILE.
;
; Some code is based on snippets from these pages:
; http://wiki.osdev.org/Setting_Up_Long_Mode
; https://os.phil-opp.com/

global krnl_start64
extern krnl_main

section .text
bits 64

; 64-bit Kernel entry point
;
; Performs long mode-related initialization and calls krnl_main.
krnl_start64:
    ; Load 0 into all data segment registers. They are not used in long mode, nevertheless
    ; it is wise to put safe values here.
    xor ax, ax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call krnl_main      ; Start real kernel
    hlt                 ; In case main exits... (shouldn't happen)
