; Kernel entry point for x86_64 arch
;
; Some code is based on snippets from these pages:
; http://wiki.osdev.org/Setting_Up_Long_Mode

section .bss
stack_bottom:
    resb 64
stack_top:

section .text
    global krnl_start
    bits 32

; Kernel entry point.
;
; Performs essential environment checks and sets up long mode.
krnl_start:
    ; initialize stack
    mov esp, stack_top

    call check_multiboot
    call check_cpuid
    call check_long_mode

    ; print `OK` to screen
    mov dword [0xb8000], 0x2f4b2f4f
    hlt

; Checks if we have proper Multiboot environment.
check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov edx, ErrNoMultiboot
    jmp error

; Checks if CPUID is supported
;
; The check is done by attempting to flip the ID bit (bit 21)
; in the FLAGS register. If we can flip it, CPUID is available.
check_cpuid:
    ; copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; copy to ECX as well for comparing later on
    mov ecx, eax

    ; flip the ID bit
    xor eax, 1 << 21

    ; copy EAX to FLAGS via the stack
    push eax
    popfd

    ; copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; restore FLAGS from the old version stored in ECX (i.e. flipping the
    ; ID bit back if it was ever flipped).
    push ecx
    popfd

    ; compare EAX and ECX. If they are equal then that means the bit
    ; wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov edx, ErrNoCpuid
    jmp error

; Checks if long mode is available.
check_long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov edx, ErrNoLongMode
    jmp error

; Kernel entry error routine.
;
; Prints (in VGA buffer) error message, pointed by [edx] and HALTS CPU.
; Stack is not used. Each error message has to end with 0x0 byte. Newlines,
; nor any other formatting, are not supported.
error:
    mov edi, 0xb8000
    cld

    ; print header, with bright red foreground color
    mov esi, ErrHeader
.head_putc:
    movsb
    mov byte [edi], 0x0C
    inc edi
    cmp byte [esi], 0
    jnz .head_putc

    ; print message, with white foreground color
    mov esi, edx
.msg_putc:
    movsb
    mov byte [edi], 0x0f
    inc edi
    cmp byte [esi], 0
    jnz .msg_putc

    ; it's time to end our journey...
    hlt

section .data
ErrHeader:          db  "Kernel boot error: ",0
ErrNoMultiboot:     db  "Attempt to boot from non-Multiboot compliant "
                    db  "bootloader.",0
ErrNoCpuid:         db  "CPUID is not supported.",0
ErrNoLongMode:      db  "Long mode is not available (current CPU is not "
                    db  "64-bit).",0
