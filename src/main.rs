#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

#[cfg(not(target_arch = "wasm32"))]
mod cli;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    match cli::main() {
        Ok(_) => {}
        Err(i) => std::process::exit(i),
    }
}
