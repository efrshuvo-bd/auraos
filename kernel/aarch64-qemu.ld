ENTRY(_start)

SECTIONS
{
  . = 0x40080000;

  .text : ALIGN(8) {
    KEEP(*(.text.boot))
    . = ALIGN(2048);
    KEEP(*(.text.vectors))
    *(.text .text.*)
  }

  .rodata : ALIGN(8) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(8) {
    *(.data .data.*)
  }

  .bss : ALIGN(16) {
    *(.bss .bss.*)
    *(COMMON)
  }
}
