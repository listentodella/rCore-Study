/* 把该文件编译,链接到.text.entry段 */
.section .text.entry

.globl _start
_start:
	/*关闭M模式的中断*/
	csrw mie, zero

	/*设置栈, 栈的大小为4KB*/
	la sp, stacks_start
	li t0, 4096
	add sp, sp, t0

	/* 跳转到 sbi_main 函数 */
	tail sbi_main

.section .data
.align 12
.globl stacks_start
stacks_start:
	.skip 4096
