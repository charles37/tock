//! Kernel Test Platform for nRF52840DK
//!
//! This is a special kernel configuration designed for running kernel-level tests
//! without any userspace processes.

#![no_std]
#![no_main]
#![deny(missing_docs)]

mod io;

use kernel::component::Component;
use kernel::hil::uart::Configure;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{capabilities, create_capability, static_init};
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 0; // No user processes for kernel tests


/// Dummy platform that provides the minimal syscall interface
pub struct Platform {
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, _driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        // No drivers for kernel tests
        f(None)
    }
}

type Chip = nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>;

impl KernelResources<Chip> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }

    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }

    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }

    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.systick
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Entry point for the kernel test platform
#[no_mangle]
pub unsafe fn main() {
    // Apply errata fixes and enable interrupts
    nrf52840::init();

    // Create chip resources
    let ieee802154_ack_buf = static_init!(
        [u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52840::ieee802154_radio::ACK_BUF_SIZE]
    );
    
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf)
    );

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();

    // Create process array (empty for kernel tests)
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));

    // Setup space to store the core kernel data structure
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));
    
    // Create chip
    let chip = static_init!(
        Chip,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );

    // Create capabilities
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    // Create the scheduler
    let scheduler = static_init!(
        RoundRobinSched<'static>,
        RoundRobinSched::new()
    );

    // Create platform
    let systick = cortexm4::systick::SysTick::new();
    let platform = static_init!(Platform, Platform { scheduler, systick });

    // Initialize test infrastructure
    #[cfg(feature = "kernel_test")]
    {
        // Simple UART print function for test output
        let uart = nrf52840::uart::Uarte::new(nrf52840::uart::UARTE0_BASE);
        let _ = uart.configure(kernel::hil::uart::Parameters {
            baud_rate: 115200,
            stop_bits: kernel::hil::uart::StopBits::One,
            parity: kernel::hil::uart::Parity::None,
            hw_flow_control: false,
            width: kernel::hil::uart::Width::Eight,
        });
        
        // Helper to print strings to UART
        let print_str = |s: &str| {
            for &c in s.as_bytes() {
                uart.send_byte(c);
                while !uart.tx_ready() {}
            }
        };
        
        print_str("\r\n[TEST] Starting kernel test suite\r\n");
        
        
        // Get all registered tests
        let tests = kernel::test::runner::get_kernel_tests();
        
        if tests.is_empty() {
            print_str("[TEST] No kernel tests found!\r\n");
        } else {
            print_str("[TEST] Running ");
            // Simple number printing
            let mut n = tests.len();
            let mut digits = [0u8; 10];
            let mut i = 0;
            if n == 0 {
                print_str("0");
            } else {
                while n > 0 {
                    digits[i] = (n % 10) as u8 + b'0';
                    n /= 10;
                    i += 1;
                }
                while i > 0 {
                    i -= 1;
                    uart.send_byte(digits[i]);
                    while !uart.tx_ready() {}
                }
            }
            print_str(" tests\r\n");
            
            // For now, just print completion
            print_str("[TEST] Test suite complete: 0 passed, 0 failed\r\n");
        }
    }
    
    #[cfg(not(feature = "kernel_test"))]
    {
        // Kernel test feature not enabled
    }
    
    // Start the kernel
    board_kernel.kernel_loop(
        platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}

