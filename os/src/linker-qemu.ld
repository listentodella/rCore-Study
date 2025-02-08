/* 要注意代码格式, 有的地方空格不能多也不能少
 * OUTPUT_ARCH 设置目标平台
 * ENTRY 设置了整个程序的入口
 * BASE_ADDRESS 这种写法是定义一个常量
 * . 号表示当前地址, 链接器会从它指向的位置开始,往下放置冲输入的目标文件中收集来的段
 * . 号既"可读", 也"可写"
 */

OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
	. = BASE_ADDRESS;
	skernel = .;

	stext = .;
	.text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        *(.text .text.*)
    }


	. = ALIGN(4K);
	etext = .;
	srodata = .;
	.rodata : {
		*(.rodata .rodata.*)
		*(.srodata .srodata.*)
	}

	. = ALIGN(4K);
	erodata = .;
	sdata = .;
	.data : {
		*(.data .data.*)
		*(.sdata .sdata.*)
	}

	. = ALIGN(4K);
	edata = .;
    sbss_with_stack = .;
	.bss : {
		*(.bss.stack)
		sbss = .;
		*(.bss .bss.*)
		*(.sbss .sbss.*)
	}

	. = ALIGN(4K);
	ebss = .;
	ekernel = .;

	/DISCARD/ : { *(.eh_frame)
	}
}
