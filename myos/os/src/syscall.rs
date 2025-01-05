use core::arch::asm;

/// 所有的syscall都是通过 ecall 指令出发
/// 约定 a0~a6保存系统调用的参数, 并且 a0 保存系统调用的返回值
/// a7 用来传递syscall ID
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a7") id
        );
    }

    ret
}

const SBI_SET_TIMER: usize = 0;
pub fn sbi_set_timer(stime_val: usize) -> isize {
    syscall(SBI_SET_TIMER, [stime_val, 0, 0])
}

const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
pub fn sbi_putchar(c: char) -> isize {
    syscall(SBI_CONSOLE_PUTCHAR, [c as usize, 0, 0])
}

pub fn sbi_put_string(str: &str) {
    str.chars().for_each(|c| {
        sbi_putchar(c);
    });
}
