// Bare-metal AArch64 boot code for Adaptive Entity Engine v1.0

.section .text.boot
.global _start

_start:
    // Set up stack pointer
    ldr x0, =_stack_top
    mov sp, x0
    
    // Clear BSS section
    ldr x0, =_bss_start
    ldr x1, =_bss_end
    mov x2, #0
    
clear_bss:
    cmp x0, x1
    b.ge clear_bss_done
    str x2, [x0], #8
    b clear_bss
    
clear_bss_done:
    // Initialize system
    bl system_init
    
    // Jump to main
    bl main
    
    // If main returns, halt
halt:
    wfi
    b halt

.section .bss
.align 16
_stack_bottom:
    .space 0x10000  // 64KB stack
_stack_top:

_bss_start:
    // BSS section start
_bss_end:
    // BSS section end
