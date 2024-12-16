use core::arch::asm;

/// sll:Shift Left Logical,最高位会被丢弃,最低位补0
/// srl:Shift Right Logical,最高位补0,最低位丢弃
/// sra:Shift Right Arithmetic, 最低位丢弃, 最高位按符号扩展
/// srai:立即数算数右移, 立即数为6位无符号数,结果需要根据rs符号扩展,然后存到rd
/// srli:立即数逻辑右移, rd = rs1 >> imm, 6位无符号数
/// slli:立即数逻辑左移, rd = rs1 << imm, 6位无符号数
pub unsafe fn test() {
    // li伪指令加载立即数
    asm!("li t0, 0x8000008a00000000");
    asm!("srai a1, t0, 1           ");
    asm!("srli t1, t0, 1           ");

    asm!("li t0, 0x128000008a      ");
    asm!("sraiw a2, t0, 1          ");
    asm!("srliw t1, t0, 1          ");
    asm!("slliw a3, t0, 1          ");

    asm!("li t0, 0x124000008a      ");
    asm!("sraiw a2, t0, 1          ");
    asm!("srliw t1, t0, 1          ");
    asm!("slliw a4, t0, 1          ");

    asm!("ret                      ");
}
