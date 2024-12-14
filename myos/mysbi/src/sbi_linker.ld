OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
	INCLUDE "src/sbi_base.ld"
}
