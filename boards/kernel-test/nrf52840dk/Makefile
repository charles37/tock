# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the kernel test platform for nRF52840DK

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=nrf52840dk-kernel-test

include ../../../boards/Makefile.common

# Default target builds the kernel
.PHONY: all
all: target/$(TARGET)/release/$(PLATFORM).bin

# Custom kernel build with test features
target/$(TARGET)/release/$(PLATFORM).elf: FORCE
	$(Q)cd ../../.. && RUSTFLAGS="-C link-arg=-T$(abspath layout.ld)" \
		cargo build --release --target=$(TARGET) --package $(PLATFORM)
	$(Q)size $@

target/$(TARGET)/release/$(PLATFORM).bin: target/$(TARGET)/release/$(PLATFORM).elf
	$(Q)$(OBJCOPY) -O binary $< $@

# Build and immediately run tests on hardware via Treadmill
.PHONY: test
test: target/$(TARGET)/release/$(PLATFORM).bin
	@echo "Kernel test binary built. Use treadmill to run on hardware."

.PHONY: clean
clean::
	$(Q)cd ../../.. && cargo clean --package $(PLATFORM)

FORCE: