// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use kernel::debug::IoWrite;
use kernel::hil::uart;
use kernel::hil::uart::Configure;
use nrf52840::uart::{Uarte, UARTE0_BASE};

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        // Here, we create a second instance of the Uarte struct.
        // This is okay because we only call this during a panic, and
        // we will never actually process the interrupts
        let uart = Uarte::new(UARTE0_BASE);
        if !self.initialized {
            self.initialized = true;
            let _ = uart.configure(uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            });
        }
        for &c in buf {
            unsafe { uart.send_byte(c) }
            while !uart.tx_ready() {}
        }
        buf.len()
    }
}

// We'll handle debug writing through the panic handler's writer
// since kernel::debug requires a more complex setup with DebugWriter

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::addr_of_mut;

    let writer = &mut *addr_of_mut!(WRITER);
    
    // Print panic info
    let _ = writer.write("PANIC: ".as_bytes());
    let _ = core::write!(writer, "{}\r\n", pi);
    
    // Infinite loop
    loop {
        cortexm4::support::nop();
    }
}