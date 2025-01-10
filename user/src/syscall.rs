// 需要内嵌汇编, 因此需要引入 core::arch::asm 模块
use core::arch::asm;

/// 所有的syscall都是通过 ecall 指令出发
/// x10 ~ x17 别名 a0 ~ a7, x1 别名 ra
/// 约定 a0~a6保存系统调用的参数, 并且 a0 保存系统调用的返回值
/// a7 用来传递syscall ID
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        /*
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
        */
        // global_asm! 宏可以将汇编代码嵌入全局
        // asm! 则可以将汇编代码嵌入到局部的函数上下文中
        // asm! 可以获取上下文中的变量信息, 并允许嵌入的汇编代码对这些变量进行操作
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

/// 功能: 将内存中缓冲区中的数据写入文件
/// syscall ID: 64
const SYSCALL_WRITE: usize = 64;
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 功能: 退出应用程序并将返回值告知批处理系统
/// syscall ID: 93
const SYSCALL_EXIT: usize = 93;
pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

const SYSCALL_YIELD: usize = 124;
pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

const SYSCALL_GET_TIME: usize = 169;
pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}
