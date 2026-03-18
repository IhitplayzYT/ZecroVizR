/*
 * Copyright (C) 2026 Ihit Rajesh Acharya
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY.
 */

// File for architecture dependent code
pub mod arch {

    #[allow(non_snake_case, non_camel_case_types, dead_code,non_upper_case_globals)]
    ///
    ///   x86 based systems test code
    ///

    #[cfg(target_arch = "x86_64")]
    pub static asm_test_code: &'static [u8; 19] = &[
        0xba, 0xf8, 0x03, /* mov $0x3f8, %dx */
        0x00, 0xd8, /* add %bl, %al */
        0x04, b'0', /* add $'0', %al */
        0xee, /* out %al, %dx */
        0xec, /* in %dx, %al */
        0xc6, 0x06, 0x00, 0x80, 0x00, /* movl $0, (0x8000); This generates a MMIO Write. */
        0x8a, 0x16, 0x00, 0x80, /* movl (0x8000), %dl; This generates a MMIO Read. */
        0xf4, /* hlt */
    ];

    ///
    ///   ARM based systems test code
    ///

    #[cfg(target_arch = "aarch64")]
    pub static asm_test_code: &'static [u8; 16] = &[
        0x01, 0x00, 0x00, 0x10, /* adr x1, <this address> */
        0x22, 0x10, 0x00, 0xb9, /* str w2, [x1, #16]; write to this page */
        0x02, 0x00, 0x00, 0xb9, /* str w2, [x0]; This generates a MMIO Write. */
        0x00, 0x00, 0x00,
        0x14, /* b <this address>; shouldn't get here, but if so loop forever */
    ];

    ///
    ///   RISCV based systems test code
    ///

    #[cfg(target_arch = "riscv64")]
    pub static asm_test_code: &'static [u8; 16] = &[
        0x17, 0x03, 0x00, 0x00, // auipc t1, 0;     <this address> -> t1
        0xa3, 0x23, 0x73, 0x00, // sw t2, t1 + 7;   dirty current page
        0x23, 0x20, 0x75, 0x00, // sw t2, a0;       trigger MMIO exit
        0x6f, 0x00, 0x00, 0x00, // j .;shouldn't get here, but if so loop forever
    ];
}
