use crate::sbi::shutdown;
use core::panic::PanicInfo;

/*
#[panic_handler] 是一种编译指导属性，用于标记核心库core中的 panic! 宏要对接的函数（该函数实现对致命错误的具体处理)
该编译指导属性所标记的函数需要具有 fn(&PanicInfo) -> ! 函数签名，函数可通过 PanicInfo 数据结构获取致命错误的相关信息
*/
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("panicked: {}", info.message());
    }

    shutdown(true)
}
