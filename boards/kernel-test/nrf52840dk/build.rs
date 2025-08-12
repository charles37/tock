// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use std::env;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    if target.starts_with("thumbv7em-none-eabi") {
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir);
        
        // Use the standard nrf52840 layout and configuration
        println!("cargo:rustc-link-search=native={}", out_path.display());
        println!("cargo:rerun-if-changed=layout.ld");
        println!("cargo:rerun-if-changed=../../../boards/nordic/nrf52840dk/layout.ld");
    }
}