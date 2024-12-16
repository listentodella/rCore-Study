use core::arch::asm;

pub unsafe fn test() {
    /* 这是一条错误指令*/
    //operand must be a symbol with %lo/%pcrel_lo/%tprel_lo modifier or an integer
    //in the range [-2048, 2047]
    //asm!("addi a1, t0, 0x800");
    asm!("addi a1, t0, 0xfffffffffffff800");
    asm!("li t0, 0x140200000             ");
    asm!("li t1, 0x40000000              ");
    asm!("addi a1, t0, 0x700             ");
    asm!("addi a1, t0, 0xfffffffffffff800");
    asm!("addiw a2, t0, 0x80             ");
    asm!("add a3, t0, t1                 ");
    asm!("addw a4, t0, t1                ");
    asm!("li t0, 0x180200000             ");
    asm!("li t1, 0x200000                ");
    asm!("sub a0, t0, t1                 ");
    asm!("subw a1, t0, t1                ");
    asm!("ret                            ");
}
