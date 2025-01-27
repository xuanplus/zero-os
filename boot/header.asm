section        .multiboot_header
header_start:
        ; magic number
        dd      0xe85250d6
        ; architecture 0 (protected mode i386)
        dd      0
        ; header length
        dd      header_end - header_start
        ; checksum
        dd      0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

        ; framebuffer tag
        dw      5       ; type
        dw      0       ; flag
        dd      20      ; size
        dd      1280    ; width
        dd      720     ; height
        dd      32      ; depth
        dd      0       ; tag end

        ; end tag
        dw      0
        dw      0
        dd      8
header_end: