/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT */
/* Copyright Tock Contributors 2023. */

MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 1M
  prog (rx) : ORIGIN = 0x00000040, LENGTH = 1M - 0x40
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

SECTIONS {
  /* Section for tests registered via macros */
  .kernel_tests : {
    __kernel_tests_start = .;
    KEEP(*(.kernel_tests))
    __kernel_tests_end = .;
  } > prog
}