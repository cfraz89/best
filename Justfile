build-all: build-demo-index-wasm build-server

build-demo-index-wasm:
  wasm-pack build demo-index --target web --out-dir ../target-wasm

build-server:
  cargo build
