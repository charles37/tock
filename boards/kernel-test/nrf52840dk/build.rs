// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use std::env;
use std::path::PathBuf;

fn main() {
    // Get the directory containing the build script
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Tell the linker where to find the layout files
    println!("cargo:rustc-link-search={}", dir.display());
    println!("cargo:rustc-link-search={}", dir.join("../../build_scripts").display());
    println!("cargo:rustc-link-arg=-Tlayout.ld");
    println!("cargo:rerun-if-changed=layout.ld");
}