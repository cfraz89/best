#![feature(exit_status_error)]
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../demo-index");
    println!("cargo:rerun-if-changed=../elementary-rs-lib");
    println!("cargo:rerun-if-changed=../elementary-rs-macros");
    let build_status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "demo-index",
            "--target-dir",
            "../target-wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .status()
        .expect("Build demo-index failed");
    build_status.exit_ok().unwrap();

    let wasm_bindgen_status = Command::new("wasm-bindgen")
        .args([
            "--target",
            "web",
            "../target-wasm/wasm32-unknown-unknown/release/demo_index.wasm",
            "--out-dir",
            "../target-wasm/pkg",
        ])
        .status()
        .expect("wasm-bindgen failed");
    wasm_bindgen_status.exit_ok().unwrap();

    let wasm_opt_status = Command::new("wasm-opt")
        .args([
            "-Os",
            "-o",
            "../target-wasm/pkg/demo_index.wasm",
            "../target-wasm/pkg/demo_index_bg.wasm",
        ])
        .status()
        .expect("wasm-opt failed");
    wasm_opt_status.exit_ok().unwrap();
}
