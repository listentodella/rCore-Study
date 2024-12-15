use core::arch::asm;

//const MY_OFFSET: i16 = -2048;

pub unsafe fn rel() {
    asm!("auipc t0, 1");
    asm!("addi t0, t0, -2048");
    asm!("ld t1, -2048(t0)");
    asm!("ret");
}
