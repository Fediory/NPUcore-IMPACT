# __clone(func, stack, flags, arg, ptid, tls, ctid)
#           a0,    a1,    a2,  a3,   a4,  a5,   a6

# syscall(SYS_clone, flags, stack, ptid, tls, ctid)
#                a7     a0,    a1,   a2,  a3,   a4

.global __clone
.type  __clone, %function
__clone:
	# Save func and arg to stack
	addi.d $a1, $a1, -16
	st.d $a0, $a1, 0
	st.d $a3, $a1, 8

	# Call SYS_clone
	move $a0, $a2
	move $a2, $a4
	move $a3, $a5
	move $a4, $a6
	li.d $a7, 220 # SYS_clone
	syscall 0

	beqz $a0, 1f
	# Parent
	jirl $r0, $r1, 0

	# Child
1:      ld.d $a1, $sp, 0
	ld.d $a0, $sp, 8
	jr $a1

	# Exit
	li.d $a7, 93 # SYS_exit
	syscall 0
