.section .text.entry
.globl   _start

_start:
	la sp, boot_stack_top

#如果在kernel早期就触发异常的话,sscratch的值为0
#与sp交换后,sp则为0,后续对sp操作基本不可能成功
#所以在这个地方给sscratch一个值是个不错的选择
#不过也不是最佳方案
#csrw sscratch, sp
call rust_main

.section .bss.stack
.globl   boot_stack_lower_bound

boot_stack_lower_bound:
	.space 4096 * 16
	.globl boot_stack_top

boot_stack_top:
