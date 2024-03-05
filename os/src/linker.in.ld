#ifdef la2k500
#define ARCH_LOONGARCH64
BASE_ADDRESS = 0x0000000000000000;
#endif

#if defined(fu740) || defined(qemu)
BASE_ADDRESS = 0x80200000;
#define ARCH_RISCV
#endif

#ifdef k210
BASE_ADDRESS = 0x80020000;
#define ARCH_RISCV
#endif

/*
*!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
*
*
*   THIS FILE WILL BE SENT TO 
*              THE  
*       GNU C PRE-PROCESSOR
*
*VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV
*>DO NOT USE THE # SYMBOL FOR COMMENTS<
*^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
*
*
*
*
*
*!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
*/

/*==========ARCHITECTURE===========*/
#ifdef ARCH_RISCV
OUTPUT_ARCH(riscv)
#endif
#ifdef ARCH_LOONGARCH64
OUTPUT_ARCH( "loongarch" )
#endif
/*==================================*/

ENTRY(_start)
SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        ssignaltrampoline = .;
        KEEP(*(.text.signaltrampoline));
#ifdef ARCH_LOONGARCH64
        . = ALIGN(4K);
        srfill = .;
        KEEP(*(.text.__rfill));
        . = ALIGN(4K);
        strap_handler = .;
        KEEP(*(.text.trap_handler));
        . = ALIGN(4K);
        skern_trap = .;
        KEEP(*(.text.kern_trap));
        . = ALIGN(4K);
#endif
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    
    . = ALIGN(4K);
    erodata = .;

#ifdef EH_ENABLED
    seh_frame_hdr = .;
    .eh_frame_hdr : {
        KEEP(*(.eh_frame_hdr .eh_frame_hdr.*))
    } 
    . = ALIGN(4K);

    eeh_frame_hdr = .;
    seh_frame = .;
    .eh_frame : {
        KEEP(*(.eh_frame .eh_frame.*))
    }
    . = ALIGN(4K);
    eeh_frame =.;
#endif

    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

#ifndef EH_ENABLED
    /DISCARD/ : {
        *(.eh_frame)
    }
#endif
}
