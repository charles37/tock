//! Kernel Test Framework
//!
//! This module provides infrastructure for writing and running tests that execute
//! within the Tock kernel. These tests can verify core kernel functionality like
//! memory protection, scheduling, and system call handling.

#![allow(dead_code)] // Test infrastructure may not be used in all builds

pub mod macros;
pub mod runner;
pub mod mpu;
pub use runner::{KernelTestRunner, TestDescriptor, TestResult};

// Note: We've removed the async test traits for simplicity
// They can be added back later if needed

/// Test assertion that includes source location information
#[macro_export]
macro_rules! kernel_test_assert {
    ($cond:expr) => {
        kernel_test_assert!($cond, "assertion failed")
    };
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return TestResult::Fail(
                crate::utilities::leasable_buffer::SubSlice::new(
                    concat!(file!(), ":", line!(), ": ", $msg).as_bytes()
                )
            );
        }
    };
}

/// Mark a test as passed
#[macro_export]
macro_rules! kernel_test_pass {
    () => {
        return TestResult::Pass
    };
}

/// Mark a test as failed with a message
#[macro_export]
macro_rules! kernel_test_fail {
    ($msg:expr) => {
        return TestResult::Fail(
            crate::utilities::leasable_buffer::SubSlice::new($msg.as_bytes())
        )
    };
}

/// Standard test output formatting
pub struct TestOutput;

impl TestOutput {
    pub fn test_start(name: &str) {
        crate::debug!("[TEST] Running {}", name);
    }
    
    pub fn test_pass(name: &str) {
        crate::debug!("[PASS] {}", name);
    }
    
    pub fn test_fail(name: &str, msg: &[u8]) {
        // Convert bytes to string for debug macro
        if let Ok(msg_str) = core::str::from_utf8(msg) {
            crate::debug!("[FAIL] {}: {}", name, msg_str);
        } else {
            crate::debug!("[FAIL] {}: (invalid UTF-8)", name);
        }
    }
    
    pub fn suite_start(count: usize) {
        crate::debug!("[TEST] Starting kernel test suite ({} tests)", count);
    }
    
    pub fn suite_complete(passed: usize, failed: usize) {
        crate::debug!("[TEST] Test suite complete: {} passed, {} failed", passed, failed);
    }
}