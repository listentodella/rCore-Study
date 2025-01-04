// 加上 .altmacro 才能正常使用 .rept 命令
.altmacro
.macro SAVE_GP n
sd     x\n, \n*8(sp)
.endm
.macro LOAD_GP n
ld     x\n, \n*8(sp)
.endm

.macro kernel_entry

// sepc + x1~x31 + sstatus + sbadaddr + scause + orig_a0 = 36
// orig_a0 is the value before syscall, just a sw backup, can remove it
addi sp, sp, -36*8
sd   x1, 1*8(sp)
sd   x3, 3*8(sp)

# save x5~x31
.set  n, 5
.rept 27
SAVE_GP %n
.set  n, n+1
.endr

csrr s1, sstatus
sd   s1, 32*8(sp)
csrr s2, sepc
sd   s1, 0*8(sp)
csrr s3, sbadaddr
sd   s3, 33*8(sp)
csrr s4, scause
sd   s4, 34*8(sp)
csrr s5, sscratch
sd   s5, 4*8(sp)

addi s0, sp, 36*8
sd   s0, 2*8(sp)
.endm

.macro kernel_exit
ld     a0, 32*8(sp)
csrw   sstatus, a0
ld     a2, 0*8(sp)
csrw   sepc, a2

ld x1, 1*8(sp)

# load x4~x31
.set  n, 4
.rept 28
SAVE_GP %n
.set  n, n+1
.endr

ld x2, 2*8(sp)

.endm

/*
   S模式的异常向量入口
   do_exception_vector必须4字节对齐
   否则写入stvec寄存器会不成功
 */
.align  2
.global do_exception_vector

do_exception_vector:
	kernel_entry
	csrw sscratch, x0
	la   ra, ret_from_exception
	mv   a0, sp                 // registers context
	mv   a1, s4
	tail do_exception

ret_from_exception:

restore_all:
	kernel_exit
	sret
