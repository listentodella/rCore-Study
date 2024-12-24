use core::arch::asm;

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
    // li 伪指令, 根据情况扩展为多条指令, 加载一个立即数
    //x[rd] = immediate
    asm!("li t0, 0x80200000");

    //x[rd] = sext(M[x[rs1] + sext(offset)][7:0])
    asm!("lb t1, (t0)   ");
    asm!("lb t1, 4(t0)  ");
    asm!("lb t1, -4(t0) ");
    asm!("ld t1, (t0)   ");
    asm!("lb t1, 4(t0)  ");

    //x[rd] = sext(immediate << 12)
    //该立即数[0, 0xFFFFF],即最多20位
    asm!("lui t0, 0x80200");
    asm!("lui t1, 0x40200");

    //x[rd] = &symbol
    asm!("la  t0, my_test_data");
    //x[rd] = &symbol
    asm!("lla t1, my_test_data");

    //pc = x[1] <<==>> jalr x0, 0(x1)
    asm!("ret                 ");
}

pub unsafe fn memcpy(src_addr: u64, dst_addr: u64, len: u64) {
    extern "C" {
        fn my_memcpy_test(src_addr: u64, dst_addr: u64, len: u64);
    }
    my_memcpy_test(src_addr, dst_addr, len);
}
