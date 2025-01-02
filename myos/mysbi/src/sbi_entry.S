//加上 .altmacro 才能正常使用 .rept 命令
.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

 /*
   sbi_exception_vector 
   M模式的异常向量入口
   8字节对齐
 */
.align 3
.global sbi_exception_vector
sbi_exception_vector:
	//从mscratch获取M模式之前备份的sp
	//并将S模式的sp保存到mscratch
	csrrw sp, mscratch, sp
	// mepc + x1~x31 + mstatus = 33
	addi sp, sp, -33*8

	sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
	sd x3, 3*8(sp)
    # skip tp(x4), application does not use it, but reserve it's mem
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
	/*保存mepc*/
	csrr t0, mepc
	sd t0, 0(sp)
	/*保存mstatus*/
	csrr t0, mstatus
	sd t0, 32*8(sp)

	/*
	   这里有两个目的:
	   1. 保存S模式的SP保存到 sbi_trap_regs->sp
	   2. 把M模式的SP保存到mscratch,
	      以便下次陷入到M模式时候可以得到SP
	 */
	//此时的SP为M模式的SP
	//mscratch保存的是S模式的SP
	addi t0, sp, 33*8
	/* 把M模式的SP保存到mscratch
	   把S模式的SP保存到 栈框sbi_trap_regs->sp里
	*/
	csrrw t0, mscratch, t0
	sd t0, 2*8(sp)

	/* 调用rust的sbi_trap_handler */
	/* sbi_trap_regs */
	mv a0, sp 
	call sbi_trap_handler










