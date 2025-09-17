//! Simple direct test output

use kernel::debug;

/// Run a simple test that outputs directly
pub fn run_simple_test() {
    // Output test messages with delays between them
    debug!("=== SIMPLE TEST START ===");
    
    // Small delay
    for _ in 0..100000 {
        cortexm4::support::nop();
    }
    
    debug!("Test message 1");
    
    // Another delay
    for _ in 0..100000 {
        cortexm4::support::nop();
    }
    
    debug!("Test message 2");
    
    // Another delay
    for _ in 0..100000 {
        cortexm4::support::nop();
    }
    
    debug!("=== SIMPLE TEST END ===");
}