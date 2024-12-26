use core::panic::PanicInfo;

use crate::base::fp::print_backtrace;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        print_backtrace();
    }
    loop {}
}
