//! Test Definition Macros
//!
//! Macros for easily defining kernel tests with minimal boilerplate.

/// Define a simple synchronous kernel test
#[macro_export]
macro_rules! kernel_test {
    (
        name: $name:ident,
        test: $body:block
    ) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub fn $name() -> $crate::test::TestResult {
            $body
        }
    };
}

/// Register kernel tests - this should be called once per module
#[macro_export]
macro_rules! register_kernel_tests {
    ($($test_name:ident),* $(,)?) => {
        // Create an array of test descriptors
        #[cfg(all(target_os = "none", not(test)))]
        #[used]
        #[link_section = ".kernel_tests"]
        pub static KERNEL_TESTS: &[$crate::test::TestDescriptor] = &[
            $(
                $crate::test::TestDescriptor {
                    name: stringify!($test_name),
                    test_fn: $crate::test::runner::TestFunction::Sync($test_name),
                },
            )*
        ];
    };
}

/// Define test assertions for kernel tests
#[macro_export]
macro_rules! assert_kernel_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            kernel_test_fail!(concat!(
                "assertion failed: ",
                stringify!($left),
                " != ",
                stringify!($right)
            ));
        }
    };
    ($left:expr, $right:expr, $msg:expr) => {
        if $left != $right {
            kernel_test_fail!($msg);
        }
    };
}

#[macro_export]
macro_rules! assert_kernel_ne {
    ($left:expr, $right:expr) => {
        if $left == $right {
            kernel_test_fail!(concat!(
                "assertion failed: ",
                stringify!($left),
                " == ",
                stringify!($right)
            ));
        }
    };
}