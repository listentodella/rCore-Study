// 把该文件编译,链接到.text.boot段
.section ".text.boot"

.globl _start

_start:
	// 关闭S模式的中断
	csrw sie, zero

	// 设置栈, 栈的大小为4KB
	la  sp, stacks_start
	li  t0, 4096
	add sp, sp, t0

	// 执行 kernel_main 函数
	tail kernel_main

hang:
	wfi
	j hang

	.section .data
	.align   12
	.globl   stacks_start

stacks_start:
	.skip 4096
