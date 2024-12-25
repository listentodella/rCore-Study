use core::arch::asm;

pub unsafe fn csrrw() {
    //csrrw rd, csr, rs
    // csr -> tmp, rs->csr, tmp->rd
    asm!("li a1, 0xFFFF");
    asm!("csrrw a0, mscratch, a1");
    asm!("li a0, 0xAAAA");
    asm!("csrrw a0, mscratch, a0");

    // 如果rd与rs相同, 则起到 scratch 与 rs交换的作用
    asm!("csrrw sp, mscratch, sp");
    // 再次调用就可以将rs备份的值取回来(前提是csr没被动过)
    asm!("csrrw sp, mscratch, sp");

    // csrw 则是伪指令, rd 其实是 x0
    // 效果是 rs->csr
    asm!("li a1, 0xBBBB");
    asm!("csrw mscratch, a1");
}
