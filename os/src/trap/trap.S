// 首先通过 __alltraps 将 Trap 上下文保存在内核栈上
// 然后跳转到使用 Rust 编写的 trap_handler 函数完成 Trap 分发及处理
// 当 trap_handler 返回之后，使用 __restore 从保存在内核栈上的 Trap 上下文恢复寄存器
// 最后通过一条 sret 指令回到应用程序执

//加上 .altmacro 才能正常使用 .rept 命令
.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
// riscv 特权级规范, 4字节对齐
.align 2
__alltraps:
    csrr tp, sstatus
    andi tp, tp, 0x100
    beqz tp, __user_trap_start
    j __real_trap_entry

__user_trap_start:
    # csrrw rd, csr, rs1
    # 控制状态寄存器读后写, 先记录csr的值t, 然后rs1存到csr, t存入rd
    # Xscratch 在异常中,提供一个字的临时存储,
    # 甚至可以当成一个普通的寄存器,如何使用完全取决于软件,硬件并不主动对它做什么
    csrrw sp, sscratch, sp #保存 sp;设置 sp 为临时内存空间的地址
__real_trap_entry:
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    # 准备在内核栈上保存trap上下文,预分配34*8字节
    # 但实际上,并没有把所有寄存器都备份了一遍
    addi sp, sp, -34*8
    # save general-purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler


__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    # mv sp, a0 # no need because __switch handle it
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # get SPP
    andi t0, t0, 0x100
    bnez t0, __kernel_trap_end

__user_trap_end:
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp # 恢复 sp;设置 mscratch 为临时内存空间的地址
    sret

__kernel_trap_end:
    #restore general-purpose registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    sret


