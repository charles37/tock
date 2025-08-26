//! Test Runner Infrastructure
//!
//! This module provides the test runner that executes all registered kernel tests.

use core::cell::Cell;
use crate::utilities::leasable_buffer::SubSlice;
use crate::deferred_call::{DeferredCall, DeferredCallClient};

use super::TestOutput;

/// Result of a test execution
pub enum TestResult {
    Pass,
    Fail(SubSlice<'static, u8>),
}

/// Descriptor for a registered test
pub struct TestDescriptor {
    pub name: &'static str,
    pub test_fn: TestFunction,
}

/// Function pointer types for different test types
pub enum TestFunction {
    Sync(fn() -> TestResult),
}

/// The main test runner
pub struct KernelTestRunner {
    tests: &'static [TestDescriptor],
    current_index: Cell<usize>,
    passed: Cell<usize>,
    failed: Cell<usize>,
    deferred_call: DeferredCall,
}

impl KernelTestRunner {
    pub fn new(tests: &'static [TestDescriptor]) -> Self {
        Self {
            tests,
            current_index: Cell::new(0),
            passed: Cell::new(0),
            failed: Cell::new(0),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Start running all tests
    pub fn run_all(&'static self) {
        TestOutput::suite_start(self.tests.len());
        self.deferred_call.set();
    }

    /// Run the next test in the sequence
    fn run_next(&self) {
        let index = self.current_index.get();
        if index >= self.tests.len() {
            TestOutput::suite_complete(self.passed.get(), self.failed.get());
            return;
        }

        let test = &self.tests[index];
        TestOutput::test_start(test.name);

        match &test.test_fn {
            TestFunction::Sync(test_fn) => {
                let result = test_fn();
                self.handle_test_result(test.name, result);
            }
        }
    }

    fn handle_test_result(&self, name: &'static str, result: TestResult) {
        match result {
            TestResult::Pass => {
                TestOutput::test_pass(name);
                self.passed.set(self.passed.get() + 1);
                self.advance_to_next_test();
            }
            TestResult::Fail(msg) => {
                TestOutput::test_fail(name, msg.as_slice());
                self.failed.set(self.failed.get() + 1);
                self.advance_to_next_test();
            }
        }
    }

    fn advance_to_next_test(&self) {
        self.current_index.set(self.current_index.get() + 1);
        self.deferred_call.set();
    }
}

impl DeferredCallClient for KernelTestRunner {
    fn handle_deferred_call(&self) {
        self.run_next();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

/// Get all registered kernel tests
/// This function collects tests from all test modules
pub fn get_kernel_tests() -> &'static [TestDescriptor] {
    // Import test arrays from modules that have them
    #[cfg(feature = "kernel_test")]
    {
        // For now, just return MPU tests
        // In a full implementation, this would merge arrays from multiple modules
        crate::test::mpu::KERNEL_TESTS
    }
    
    #[cfg(not(feature = "kernel_test"))]
    {
        &[]
    }
}