global          start
extern          long_mode_start

section         .text
bits            32
start:
        ; update stack pointer
        mov     esp,    stack_top

        ; pass arguments to rust kernel
        mov     edi,    ebx
        mov     esi,    eax

        ; check something (from osdev wiki)
        call    check_multiboot
        call    check_cpuid
        call    check_long_mode

        ; setup and enable paging
        call    set_up_page_tables
        call    enable_paging

        ; go to long
        lgdt    [gdt64.pointer]
        jmp     gdt64.code:long_mode_start

        hlt

check_multiboot:
        cmp     eax,    0x36d76289
        jne     .no_multiboot
        ret

.no_multiboot:
        mov     al,     "0"
        jmp     error

; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
; in the FLAGS register. If we can flip it, CPUID is available.
check_cpuid:
        ; Copy FLAGS in to EAX via stack
        pushfd
        pop     eax

        ; Copy to ECX as well for comparing later on
        mov     ecx,    eax

        ; Flip the ID bit
        xor     eax,    1 << 21

        ; Copy EAX to FLAGS via the stack
        push eax
        popfd

        ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
        pushfd
        pop     eax

        ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
        ; ID bit back if it was ever flipped).
        push    ecx
        popfd

        ; Compare EAX and ECX. If they are equal then that means the bit
        ; wasn't flipped, and CPUID isn't supported.
        cmp     eax,    ecx
        je      .no_cpuid
        ret

.no_cpuid:
        mov     al,     "1"
        jmp     error

; Test if extended processor info in available
check_long_mode:
        ; implicit argument for cpuid
        mov     eax,    0x80000000
        ; get highest supported argument
        cpuid
        ; it needs to be at least 0x80000001
        cmp     eax,    0x80000001
        ; if it's less, the CPU is too old for long mode
        jb      .no_long_mode

        ; use extended info to test if long mode is available
        ; argument for extended processor info
        mov     eax,    0x80000001
        ; returns various feature bits in ecx and edx
        cpuid
        ; test if the LM-bit is set in the D-register
        test    edx,    1 << 29
        ; If it's not set, there is no long mode
        jz      .no_long_mode
        ret

.no_long_mode:
        mov     al, "2"
        jmp     error

set_up_page_tables:
        ; map first P4 entry to P3 table
        mov     eax,            p3_table
        or      eax,            0b11    ; present + writable
        mov     [p4_table],     eax

        ; map first five P3 entries to P2 table
        mov     eax,            p2_table + 0x1000 * 0
        or      eax,            0b11
        mov     [p3_table],     eax

        mov     eax,            p2_table + 0x1000 * 1
        or      eax,            0b11
        mov     [p3_table+8], eax

        mov     eax,            p2_table + 0x1000 * 2
        or      eax,            0b11
        mov     [p3_table+16],  eax

        mov     eax,            p2_table + 0x1000 * 3
        or      eax,            0b11
        mov     [p3_table+24],  eax

        mov     eax,            p2_table + 0x1000 * 4
        or      eax,            0b11
        mov     [p3_table+32],  eax

        ; map each P2 entry to a huge 2MiB page
        mov ecx, 0      ; counter variable

.map_p2_table:
        ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
        mov     eax,    0x200000                ; 2MiB
        mul     ecx                             ; start address of ecx-th page
        or      eax,    0b10000011              ; present + writable + huge
        mov     [p2_table+ecx*8],       eax     ; map ecx-th entry

        inc     ecx                             ; increase counter
        cmp     ecx,    512 * 5                 ; if counter == 512, the whole P2 table is mapped
        jne     .map_p2_table                   ; else map the next entry

        ret

enable_paging:
        ; load P4 to cr3 register (cpu uses this to access the P4 table)
        mov     eax,    p4_table
        mov     cr3,    eax

        ; enable PAE-flag in cr4 (Physical Address Extension)
        mov     eax,    cr4
        or      eax,    1 << 5
        mov     cr4,    eax

        ; set the long mode bit in the EFER MSR (model specific register)
        mov     ecx,    0xC0000080
        rdmsr
        or      eax,    1 << 8
        wrmsr

        ; enable paging in the cr0 register
        mov     eax,    cr0
        or      eax,    1 << 31
        mov     cr0,    eax

        ret

; Prints `ERR: ` and the given error code to screen and hangs.
; parameter: error code (in ascii) in al
error:
        mov     dword [0xb8000],        0x4f524f45
        mov     dword [0xb8004],        0x4f3a4f52
        mov     dword [0xb8008],        0x4f204f20
        mov     byte  [0xb800a],        al
        hlt

section .bss
align 4096
p4_table:
        resb    4096
p3_table:
        resb    4096
p2_table:
        resb    4096 * 5
stack_bottom:
        resb    64
stack_top:

section .rodata
gdt64:
        dq      0 ; zero entry
.code: equ      $ - gdt64 ; new
        dq      (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
        dw $ - gdt64 - 1
        dq      gdt64