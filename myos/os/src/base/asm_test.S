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


