// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Hardware Test Framework
//!
//! This module provides a simpler way to write hardware tests that run
//! directly in the kernel without requiring Python harnesses.

use crate::debug;

/// Trait that hardware tests must implement
pub trait HardwareTest {
    /// Name of the test for reporting
    fn name(&self) -> &'static str;
    
    /// Boards this test supports (empty = all boards)
    fn supported_boards(&self) -> &'static [&'static str] {
        &[] // Default: runs on all boards
    }
    
    /// Run the test, returning Ok(()) on success
    fn run(&self) -> Result<(), &'static str>;
}

/// Test result reporting
#[derive(Debug)]
pub enum TestResult {
    Pass,
    Fail(&'static str),
    Skip(&'static str),
}

/// Hardware test runner that manages test execution
pub struct HardwareTestRunner {
    tests: &'static [&'static dyn HardwareTest],
    current_board: &'static str,
}

impl HardwareTestRunner {
    pub fn new(tests: &'static [&'static dyn HardwareTest], board: &'static str) -> Self {
        Self {
            tests,
            current_board: board,
        }
    }
    
    pub fn run_all(&self) {
        debug!("=== Hardware Test Suite Starting ===");
        debug!("Board: {}", self.current_board);
        debug!("Tests: {}", self.tests.len());
        debug!("");
        
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        
        for test in self.tests {
            let result = self.run_single_test(*test);
            match result {
                TestResult::Pass => passed += 1,
                TestResult::Fail(_) => failed += 1,
                TestResult::Skip(_) => skipped += 1,
            }
        }
        
        debug!("");
        debug!("=== Test Summary ===");
        debug!("Passed:  {}", passed);
        debug!("Failed:  {}", failed);
        debug!("Skipped: {}", skipped);
        debug!("Total:   {}", self.tests.len());
        
        if failed == 0 {
            debug!("=== All tests passed! ===");
        } else {
            debug!("=== Tests FAILED ===");
        }
    }
    
    fn run_single_test(&self, test: &dyn HardwareTest) -> TestResult {
        debug!("Running: {}", test.name());
        
        // Check if test supports this board
        let supported = test.supported_boards();
        if !supported.is_empty() && !supported.contains(&self.current_board) {
            debug!("  SKIP: Not supported on {}", self.current_board);
            return TestResult::Skip("Board not supported");
        }
        
        // Run the test
        match test.run() {
            Ok(()) => {
                debug!("  PASS");
                TestResult::Pass
            }
            Err(msg) => {
                debug!("  FAIL: {}", msg);
                TestResult::Fail(msg)
            }
        }
    }
}

/// Macro to simplify test registration
#[macro_export]
macro_rules! hardware_test {
    ($name:ident, $test_fn:expr) => {
        struct $name;
        impl $crate::test::hardware::HardwareTest for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }
            
            fn run(&self) -> Result<(), &'static str> {
                $test_fn()
            }
        }
    };
    
    ($name:ident, boards = [$($board:expr),*], $test_fn:expr) => {
        struct $name;
        impl $crate::test::hardware::HardwareTest for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }
            
            fn supported_boards(&self) -> &'static [&'static str] {
                &[$($board),*]
            }
            
            fn run(&self) -> Result<(), &'static str> {
                $test_fn()
            }
        }
    };
}

/// Macro to create test suite
#[macro_export]
macro_rules! create_hardware_test_suite {
    ($($test:ident),*) => {
        pub fn hardware_tests() -> &'static [&'static dyn $crate::test::hardware::HardwareTest] {
            &[
                $(&$test as &dyn $crate::test::hardware::HardwareTest),*
            ]
        }
    };
}