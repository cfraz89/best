use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../demo-index");
    println!("cargo:rerun-if-changed=../elementary-rs-lib");
    println!("cargo:rerun-if-changed=../elementary-rs-macros");
    Command::new("cargo")
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
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Command::new("wasm-bindgen")
        .args([
            "--target",
            "web",
            "../target-wasm/wasm32-unknown-unknown/release/demo_index.wasm",
            "--out-dir",
            "../target-wasm/pkg",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Command::new("wasm-opt")
        .args([
            "-Os",
            "-o",
            "../target-wasm/pkg/demo_index.wasm",
            "../target-wasm/pkg/demo_index_bg.wasm",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}