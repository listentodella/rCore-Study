.align 3
.globl my_test_data

my_test_data:
	.dword 0x12345678abcdabcd

	.global load_store_test
	.global compare_and_return

load_store_test:
	li t0, 0x80200000

	lb t1, (t0)
	lb t1, 4(t0)
	lb t1, -4(t0)
	ld t1, (t0)
	lb t1, 4(t0)

	lui t0, 0x80200
	lui t1, 0x40200

	la  t0, my_test_data
	lla t1, my_test_data

	ret

.global my_memcpy_test
my_memcpy_test:
	// mv在RV里是伪指令
	// 等价于 addi rd, rs, 0
	mv t0, a0
	mv t1, a1
	add t2, t0, a2

.loop:
	// RV 并没有ARM的多字节加载指令 
	ld t3, (t0)
	sd t3, (t1)
	addi t0, t0, 8
	addi t1, t1, 8
	blt t0, t2, .loop

	ret


compare_and_return:
    bltu a0, a1, .L2
    li a5, 0
    j .L3
.L2:
    li a5, -1
.L3:
    mv a0, a5
    ret

.global beqz_test
beqz_test:
	beqz a0, .L4
	li a5, 0
	j .L5
.L4:
	li a5, 1
.L5:
	mv a0, a5
	ret


.globl add_test
add_test:
	add a0, a0, a1
	nop
	// 加载它的父函数的返回地址
	ret

.globl branch_test
branch_test:
	/*把返回地址ra寄存器保存到栈里*/
	addi	sp,sp,-8
	// 此时的ra指向父函数
	sd	ra,(sp)
	// 执行完成后,将记录返回地址(父函数)保存到sp

	li a0, 1
	li a1, 2
	/* 调用add_test子函数 */
	// call是伪指令, 本质是auipc与jalr的组合
	// 将下一条指令的地址(pc+8)写入 x[rd],然后将 pc 设为 symbol
	// 它并不会主动为我们备份当前的ra, 
	// 不过会将当前的运行函数备份到ra(具体要看编译器选了哪个寄存器备份)
	// 所以我们需要在上面自己备份父函数的返回地址到ra
	call add_test
	nop

	/* 从栈中恢复ra返回地址*/
	ld	ra,(sp)
	addi	sp,sp,8
	ret
