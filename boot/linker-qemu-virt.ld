ENTRY(_start)

SECTIONS
{
  . = 0x40080000;

  .text : {
    KEEP(*(.text.boot))
    *(.text .text.*)
  }

  .rodata : {
    *(.rodata .rodata.*)
  }

  .data : {
    *(.data .data.*)
  }

  .bss : {
    *(.bss .bss.*)
    *(COMMON)
  }

  /DISCARD/ : {
    *(.eh_frame*)
  }
}
