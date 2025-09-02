//! Kernel Test Platform for nRF52840DK
//!
//! This is a special kernel configuration designed for running kernel-level tests
//! without any userspace processes.

#![no_std]
#![no_main]
#![deny(missing_docs)]

mod io;
mod test_launcher;
mod simple_test;

use kernel::component::Component;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{capabilities, create_capability, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use nrf52_components::{UartChannel, UartPins};
use test_launcher::TestLauncher;

// UART pin configuration
const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 0; // No user processes for kernel tests

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

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

    // Set up timer for UART
    let rtc = &nrf52840_peripherals.nrf52.rtc;
    let _ = rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(nrf52840::rtc::Rtc));

    // Set up UART channel
    let uart_channel = nrf52_components::UartChannelComponent::new(
        UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD)),
        mux_alarm,
        &nrf52840_peripherals.nrf52.uarte0,
    )
    .finalize(nrf52_components::uart_channel_component_static!(
        nrf52840::rtc::Rtc
    ));

    // Create UART mux for multiplexing UART
    let uart_mux = components::console::UartMuxComponent::new(uart_channel, 115200)
        .finalize(components::uart_mux_component_static!());

    // Create the debugger object that handles calls to `debug!()`
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());
    
    // Initialize test infrastructure
    #[cfg(feature = "kernel_test")]
    {
        // Add a small delay to ensure UART is ready
        for _ in 0..1000000 {
            cortexm4::support::nop();
        }
        
        // Run simple test first
        simple_test::run_simple_test();
        
        // Output a simple test message directly
        kernel::debug!("=== NRF52840DK Kernel Test Starting ===");
        
        // Create test launcher
        let test_launcher = static_init!(
            TestLauncher,
            TestLauncher::new()
        );
        
        // Start tests before entering kernel loop
        test_launcher.start();
        
        // Start the kernel loop which will handle test execution
        board_kernel.kernel_loop(
            platform,
            chip,
            None::<&kernel::ipc::IPC<0>>,
            &main_loop_capability,
        );
    }
    
    #[cfg(not(feature = "kernel_test"))]
    {
        // Normal kernel operation - start the kernel loop
        board_kernel.kernel_loop(
            platform,
            chip,
            None::<&kernel::ipc::IPC<0>>,
            &main_loop_capability,
        );
    }
}

