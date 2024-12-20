use core::arch::asm;

//const MY_OFFSET: i16 = -2048;

pub unsafe fn rel() {
    //x[rd] = pc + sext(immediate << 12)
    //该立即数[0, 0xFFFFF],即最多20位
    asm!("auipc t0, 1");

    //x[rd] = x[rs1] + sext(immediate)
    asm!("addi t0, t0, -2048");

    //x[rd] = M[x[rs1] + sext(offset)][63:0]
    asm!("ld t1, -2048(t0)");
    asm!("ret");
}
