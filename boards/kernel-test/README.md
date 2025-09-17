# Kernel Test Infrastructure

This directory contains board configurations for running kernel-level tests without userspace processes.

## Overview

The kernel test infrastructure allows testing core kernel functionality (MPU, scheduling, etc.) directly without requiring userspace. Tests are compiled into the kernel and run automatically on boot.

## Structure

- `nrf52840dk/` - Test kernel configuration for nRF52840 Development Kit
  - Additional board configurations can be added here

## Test Framework Features

- **Macro-based test registration**: Tests self-register using linker sections
- **Synchronous and asynchronous tests**: Support for both test patterns
- **Structured output**: Consistent `[TEST]`, `[PASS]`, `[FAIL]` format
- **No manual indexing**: Tests are automatically discovered

## Writing Tests

Tests are defined in `kernel/src/test/` using the provided macros:

```rust
// Simple synchronous test
kernel_test! {
    name: test_example,
    test: {
        assert_kernel_eq!(2 + 2, 4);
        kernel_test_pass!();
    }
}

// Asynchronous test
async_kernel_test! {
    name: test_async_example,
    state: MyTestState,
    setup: |test| { /* initialization */ },
    test: |test| { /* test logic */ },
    client_field: client
}
```

## Running Tests

### Local Development

```bash
cd boards/kernel-test/nrf52840dk
make
# Flash the resulting binary to your board
```

### With Treadmill CI

```python
# In hwci/tests/kernel_mpu_tests.py
class KernelMpuTest(KernelTestHarness):
    KERNEL_CONFIG = "kernel-test"
    # Test implementation...
```

## Adding New Tests

1. Create test module in `kernel/src/test/`
2. Use the test macros to define tests
3. Add module to `kernel/src/test/mod.rs`
4. Tests will automatically be included in the kernel-test build

## Test Output Format

```
[TEST] Starting kernel test suite (8 tests)
[TEST] Running test_mpu_basic_configuration
[PASS] test_mpu_basic_configuration
[TEST] Running test_mpu_region_boundaries
[PASS] test_mpu_region_boundaries
...
[TEST] Test suite complete: 8 passed, 0 failed
```