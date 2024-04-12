.section .text.entry
.globl _start
_start:
    move $a0, $sp
    bl __start_main
