    .section .text.entry
    .globl _start
_start:
    pcaddi      $t0,    0x0
    srli.d      $t0,    $t0,    0x30
    slli.d      $t0,    $t0,    0x30
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x181   # Make sure the window remains the same after the switch.
    sub.d       $t0,    $t0,    $t0
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x180
    pcaddi      $t0,    0x0
    slli.d      $t0,    $t0,    0x10
    srli.d      $t0,    $t0,    0x10
    jirl        $t0,    $t0,    0x10    # 跳0段的下一条指令
    # The barrier
    sub.d       $t0,    $t0,    $t0
    csrwr       $t0,    0x181
    sub.d       $t0,    $t0,    $t0
    la.global $sp, boot_stack_top
    bl          rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
