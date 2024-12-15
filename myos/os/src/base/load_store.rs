use core::arch::asm;
use core::arch::global_asm;

global_asm!(include_str!("asm_test.S"));

pub unsafe fn global_asm_test() {
    extern "C" {
        fn load_store_test();
    }

    load_store_test();
}

pub unsafe fn asm_all() {
    asm!(
        "
        li t0, 0x80200000

        lb t1, (t0)
        lb t1, 4(t0)
        lb t1, -4(t0)
        ld t1, (t0)
        lb t1, 4(t0)

        lui t0, 0x80200
        lui t1, 0x40200

        ret
"
    );
}

pub unsafe fn asm_single() {
    //     asm!(
    //         "
    // .align 3
    // .globl my_test_data
    // my_test_data:
    // 	.dword 0x12345678abcdabcd
    // "
    //     );
    //let my_test_data = 0x12345678abcdabcdu64;

    asm!("li t0, 0x80200000");
    asm!("lb t1, (t0)   ");
    asm!("lb t1, 4(t0)  ");
    asm!("lb t1, -4(t0) ");
    asm!("ld t1, (t0)   ");
    asm!("lb t1, 4(t0)  ");
    asm!("lui t0, 0x80200");
    asm!("lui t1, 0x40200");
    asm!("la  t0, my_test_data");
    asm!("lla t1, my_test_data");
    asm!("ret                 ");
}
