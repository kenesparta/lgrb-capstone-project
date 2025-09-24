MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Add defmt section for proper logging support */
SECTIONS
{
  .defmt : {
    *(.defmt .defmt.*)
  } > FLASH
}