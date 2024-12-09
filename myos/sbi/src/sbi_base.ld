BASE_ADDRESS = 0x80000000;

. = BASE_ADDRESS;

.text.boot : { *(.text.boot) }
.text : { *(.text) }
.rodata : { *(.rodata) }
.data : { *(.data) }
. = ALIGN(0x08);
bss_start = .;
.bss : { *(.bss*) }
bss_end = .;

