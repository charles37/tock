//! Kernel Test Platform for nRF52840DK
//!
//! This is a special kernel configuration designed for running kernel-level tests
//! without any userspace processes.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::panic::PanicInfo;

use kernel::component::Component;
use kernel::debug;
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
        use kernel::test::{TestOutput, KernelTestRunner};
        
        // Get all registered tests
        let tests = kernel::test::runner::get_kernel_tests();
        
        TestOutput::suite_start(tests.len());
        
        if tests.is_empty() {
            TestOutput::suite_complete(0, 0);
            debug!("No kernel tests found!");
        } else {
            // Create and run test runner
            let test_runner = static_init!(
                KernelTestRunner,
                KernelTestRunner::new(tests)
            );
            
            test_runner.register();
            test_runner.run_all();
        }
    }
    
    #[cfg(not(feature = "kernel_test"))]
    {
        debug!("Kernel test feature not enabled!");
    }
    
    // Start the kernel
    board_kernel.kernel_loop(
        platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}

/// Panic handler
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    debug!("\n[KERNEL PANIC] Test kernel panic:\n{}\n", info);
    
    // For kernel tests, we want to see the panic
    cortexm4::support::nop();
    
    loop {
        cortexm4::support::nop();
    }
}