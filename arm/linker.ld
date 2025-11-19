/* Linker script for bare-metal AArch64 */

ENTRY(_start)

SECTIONS
{
    . = 0x40000000;  /* Typical AArch64 entry point */
    
    .text : {
        *(.text.boot)
        *(.text*)
    }
    
    .rodata : {
        *(.rodata*)
    }
    
    .data : {
        *(.data*)
    }
    
    .bss : {
        _bss_start = .;
        *(.bss*)
        _bss_end = .;
    }
}
