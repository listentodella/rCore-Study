// pub fn console_putchar(c: usize) {
//     #[allow(deprecated)]
//     sbi_rt::legacy::console_putchar(c);
// }

// pub fn shutdown(failure: bool) -> ! {
//     use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
//     if !failure {
//         system_reset(Shutdown, NoReason);
//     } else {
//         system_reset(Shutdown, SystemFailure);
//     }

//     unreachable!()
// }

const UART_BASE_ADDR: usize = 0x10000000;
/* THR:transmitter holding register */
const UART_DAT: usize = UART_BASE_ADDR + 0x00; /* 数据寄存器*/
const UART_IER: usize = UART_BASE_ADDR + 0x01; /* 中断使能寄存器*/
const UART_IIR: usize = UART_BASE_ADDR + 0x02; /* 中断标识寄存器 (read only)*/
const UART_FCR: usize = UART_BASE_ADDR + 0x02; /* FIFO控制寄存器 (write only)*/
const UART_LCR: usize = UART_BASE_ADDR + 0x03; /* 线路控制寄存器*/
const UART_MCR: usize = UART_BASE_ADDR + 0x04; /* MODEN控制寄存器*/
const UART_LSR: usize = UART_BASE_ADDR + 0x05; /* 线路状态寄存器*/
const UART_MSR: usize = UART_BASE_ADDR + 0x06; /* MODEN状态寄存器*/

const UART_DLL: usize = UART_BASE_ADDR + 0x00; /*预分频寄存器低8位*/
const UART_DLM: usize = UART_BASE_ADDR + 0x01; /*预分频寄存器高8位*/

const UART_LSR_ERROR: usize = 0x80; /* 出错 */
const UART_LSR_EMPTY: u8 = 0x40; /* 传输FIFO和移位寄存器为空 */
const UART_LSR_TFE: usize = 0x20; /* 传输FIFO为空 */
const UART_LSR_BI: usize = 0x10; /* 传输被打断 */
const UART_LSR_FE: usize = 0x08; /* 接收到没有停止位的帧 */
const UART_LSR_PE: usize = 0x04; /* 奇偶校验错误位 */
const UART_LSR_OE: usize = 0x02; /* 数据溢出 */
const UART_LSR_DR: usize = 0x01; /* FIFO有数据 */

const UART_DEFAULT_BAUD: usize = 115200;
const UART_16500_CLK: usize = 1843200;

pub fn init() {
    let divisor = UART_16500_CLK / (16 * UART_DEFAULT_BAUD);

    unsafe {
        // disable uart irq
        (UART_IER as *mut u8).write_volatile(0);

        // enable DLAB for baud settings
        (UART_LCR as *mut u8).write_volatile(0x80);
        (UART_DLL as *mut u8).write_volatile(divisor as u8);
        (UART_DLM as *mut u8).write_volatile((divisor >> 8) as u8);

        // set uart format
        /*8 bits, no parity, one stop bit*/
        (UART_LCR as *mut u8).write_volatile(0x03u8);

        // enable fifo, clear fifo, and watermark at 14B
        (UART_FCR as *mut u8).write_volatile(0xc7u8);
    }
}

pub fn putchar(c: usize) {
    unsafe {
        while (UART_LSR as *const u8).read_volatile() & UART_LSR_EMPTY == 0 {}
        (UART_DAT as *mut u8).write_volatile(c as u8);
    }
}
