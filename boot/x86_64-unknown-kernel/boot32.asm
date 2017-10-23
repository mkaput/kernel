; Kernel entry point for x86_64 arch, 32-bit initialization code
;
; THERE ARE NO CALLING CONVENTIONS IN THIS FILE.

global krnl_start32
extern krnl_start64

%define MULTIBOOT_MAGIC 0x36d76289

section .bss
align 4096
; Page tables
;
; In long mode, x86 uses a page size of 4096 bytes and a 4 level page table that consists of:
; P4) the Page-Map Level-4 Table (PML4),
; P3) the Page-Directory Pointer Table (PDP),
; P2) the Page-Directory Table (PD),
; P1) and the Page Table (PT).
;
; In the x86 architecture, the page tables are hardware walked, so the CPU will look at the table
; on its own when it needs a translation.
;
; Page tables need to be page-aligned as the bits 0-11 are used for flags. By putting these tables
; at the beginning of .bss, the linker can just page align the whole section and we don't have
; unused padding bytes in between.
P4_TABLE:   resb 4096
P3_TABLE:   resb 4096
P2_TABLE:   resb 4096

; Stack
;
; We will initially use simple 64 byte stack for booting use. It will be resized further.
STACK_BOTTOM:
    resb 64
STACK_TOP:

section .text
bits 32

; Kernel entry point
;
; Performs essential environment checks, sets up long mode and far jumps to 64-bit initialization
; code.
krnl_start32:
    ; Initialize basic stack
    mov esp, STACK_TOP

    ; Run basic system checks and initialization
    call check_multiboot
    call check_cpuid
    call check_long_mode

    call init_page_tables
    call enable_paging

    ; Load the 64-bit GDT
    lgdt [GDT64.pointer]

    ; Go to 64-bit code
    jmp GDT64.code:krnl_start64

; Checks if we have proper Multiboot environment
;
; Multiboot writes magic 0x36d76289 to EAX before jumping to kernel code and presence of this
; value is checked. Therefore nothing should touch EAX before this procedure call.
check_multiboot:
    cmp eax, MULTIBOOT_MAGIC
    jne .no_multiboot
    ret
.no_multiboot:
    mov edx, ERR_NO_MULTIBOOT
    jmp error

; Checks if CPUID is available
;
; The check is done by attempting to flip the ID bit (bit 21)
; in the FLAGS register. If we can flip it, CPUID is available.
check_cpuid:
    ; Copy FLAGS into EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit back if it was
    ; Ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit wasn't flipped,
    ; and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov edx, ERR_NO_CPUID
    jmp error

; Checks if long mode is available.
check_long_mode:
    ; Test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; Use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov edx, ERR_NO_LONG_MODE
    jmp error

; Sets up initial page tables
;
; These initial tables map directly to physical memory (identity paging).
; First 1GiB of kernel is mapped (because it's straightforward to do it shortly in asm).
init_page_tables:
    ; Map first P4 entry to P3 table
    mov eax, P3_TABLE
    or eax, 0b000000000011      ; present + writable
    mov dword [P4_TABLE], eax

    ; Map first P3 entry to P2 table
    mov eax, P2_TABLE
    or eax, 0b000000000011      ; present + writable
    mov dword [P3_TABLE], eax

    ; Map each P2 entry to a huge 2MiB page
    xor ecx, ecx

.map_p2_table:
    ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
    mov eax, 0x200000           ; 2MiB
    mul ecx                     ; start address of ecx-th page
    or eax, 0b000010000011      ; present + writable + huge page
    mov dword [P2_TABLE + ecx * 8], eax

    inc ecx
    cmp ecx, 512                ; if ecx == 512, then whole P2 table is mapped
    jne .map_p2_table

    ret

; Enables paging and enters the 32-bit compatibility submode of long mode
enable_paging:
    ; Write the address of the P4 table to the cr3 register CPU will look there for it
    mov eax, P4_TABLE
    mov cr3, eax

    ; Enable PAE-paging by setting the PAE-bit (6th bit) in cr4
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Set the long mode bit (9th bit in EFER MSR)
    mov ecx, 0xC0000080          ; set ecx to 0xC0000080, which is the EFER MSR
    rdmsr
    or eax, 1 << 8               ; set the long mode bit
    wrmsr

    ; Enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31              ; set paging bit (32nd bit of cr0)
    mov cr0, eax

    ret

; Kernel entry error routine.
;
; Prints (in VGA buffer) error message, pointed by [edx] and HALTS CPU.
; Stack is not used. Each error message has to end with 0x0 byte. Newlines,
; nor any other formatting, are not supported.
error:
    mov edi, 0xb8000
    cld

    ; Print header, with bright red foreground color
    mov esi, ERR_HEAD
.head_putc:
    movsb
    mov byte [edi], 0x0C
    inc edi
    cmp byte [esi], 0
    jnz .head_putc

    ; Print message, with white foreground color
    mov esi, edx
.msg_putc:
    movsb
    mov byte [edi], 0x0f
    inc edi
    cmp byte [esi], 0
    jnz .msg_putc

    ; It's time to end our journey...
    hlt

section .rodata
; Global Descriptor Table (64-bit)
;
; It's used for segmentation, though we won't use segementation.
; But it's required anyway so we do it.
GDT64:              dq 0 ; zero entry
.code:              equ $ - GDT64   ; code segment
                    dq (1<<43) | (1<<44) | (1<<47)  | (1<<53)
                       ; 43 - executable
                       ; 44 - descriptor type (should be 1 for code and data segments)
                       ; 47 - present (must be 1 for valid selectors)
                       ; 53 - 64-bit (should be set for 64-bit code segments)
.pointer:           dw $ - GDT64 - 1
                    dq GDT64

; Error messages
ERR_HEAD:           db  "Kernel boot error: ",0
ERR_NO_MULTIBOOT:   db  "Attempt to boot from non-Multiboot compliant bootloader.",0
ERR_NO_CPUID:       db  "CPUID is not supported.",0
ERR_NO_LONG_MODE:   db  "Long mode is not available (current CPU is not 64-bit).",0
