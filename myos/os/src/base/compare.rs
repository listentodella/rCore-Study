use core::arch::{asm, naked_asm};

/// bltu:无符号小于时分支跳转
/// bltu rs1, rs2, offset if (rs1 < rs2) ...
pub unsafe fn is_little_than(a: u64, b: u64) -> bool {
    // 不论是拆分成多句还是合并成一段
    // rust编译器都不接受label名称
    // asm!("bltu a0, a1, .L2");
    // asm!("li a5, 0");
    // asm!("j .L3");
    // asm!(".L2:");
    // asm!("li a5, -1");
    // asm!(".L3:");
    // asm!("mv a0, a5");
    // asm!("ret");

    // 接受纯数字的label名称
    // 也接受数字打头的名称(但是文档建议只用纯数字)
    // https://rustwiki.org/en/rust-by-example/unsafe/asm.html
    let x: i8;
    // asm!("bltu {0}, {1}, 2", in(reg) a, in(reg) b);
    // asm!("li a5, 0");
    // asm!("j 4");
    // asm!("2:");
    // asm!("li a5, -1");
    // asm!("4:");
    // asm!("mv {}, a5", out(reg) x);
    //FIXME: how to add labels correctly in rust asm?
    // asm!(
    //     "bltu {0}, {1}, 2",
    //     "li a5, 0",
    //     "j 4",
    //     "2:",
    //     "li a5, -1",
    //     "4:",
    //     "mv {2}, a5",
    //     in(reg) a, in(reg) b,
    //     out(reg) x);
    //asm!("ret");
    extern "C" {
        fn compare_and_return(a: u64, b: u64) -> i8;
    }
    x = compare_and_return(a, b);

    if x == -1 {
        true
    } else {
        false
    }
}

// naked_asm, 真正意义上的裸函数
// 只能有一个,并且无法接收参数, 也无法带出返回值
// 但是单步依旧无法直接在gdb上看source的状态,只能看assembly :(
#[naked]
//#[warn(undefined_naked_function_abi)]
// 使用rust的函数签名已经没有意义
//pub unsafe fn naked_is_little_than() {
pub unsafe extern "C" fn naked_is_little_than() {
    naked_asm!(
        "li a0, 100",
        "li a1, 1000",
        "bltu a0, a1, 2",
        "li a5, 0",
        "j 4",
        "2:",
        "li a5, -1",
        "4:",
        "mv a0, a5",
        "ret"
    );
}

pub unsafe fn is_zero(a: u64) -> bool {
    extern "C" {
        fn beqz_test(a: u64) -> u8;
    }
    let x = beqz_test(a);
    if x == 0 {
        false
    } else {
        true
    }
}
