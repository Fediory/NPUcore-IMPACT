    .section .text.entry
    .globl _start
_start:
    # 1. Let t0 <- PC
    pcaddi      $t0,    0x0

    # 2. Let t0 = 0x9000_0000_0000_0000
    srli.d      $t0,    $t0,    0x30
    slli.d      $t0,    $t0,    0x30
    
    # 3. Switch CSR_reg(DMWIN1) and (t0 + 0x11), PLV0 for 0x0 ~ 0xFFFF_FFFF_FFFF_FFFF
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x181   # Make sure the window remains the same after the switch.
    sub.d       $t0,    $t0,    $t0
    
    # 4. Switch CSR_reg(DMWIN0) and (t0 + 0x11), ditto
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x180
    sub.d       $t0,    $t0,    $t0

    # 5. Set t0 = 0x0000_0000_9000_0038, and jump to the barrier
    pcaddi      $t0,    0x0
    slli.d      $t0,    $t0,    0x10
    srli.d      $t0,    $t0,    0x10
    jirl        $t0,    $t0,    0x10  
      
    # 6. Switch CSR_reg(number = 181) = 0
    sub.d       $t0,    $t0,    $t0
    csrwr       $t0,    0x181
    sub.d       $t0,    $t0,    $t0


    la.global   $sp, boot_stack_top
    bl          rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
