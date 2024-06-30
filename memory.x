MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
    /* Normal setup is 256K:
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K

    But with self-debug, we need to use less:
    */
    RAM   : ORIGIN = 0x20000000, LENGTH = 240K
}

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2
} INSERT BEFORE .text;