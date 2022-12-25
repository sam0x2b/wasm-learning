@echo off

cargo build --target wasm32-unknown-unknown --release
cd target/wasm32-unknown-unknown/release
wasm-bindgen --target web --no-typescript --out-dir . wasm_sample.wasm
wasm-gc wasm_sample_bg.wasm
cd ../../../www
copy /y ..\target\wasm32-unknown-unknown\release\wasm_sample.js
copy /y ..\target\wasm32-unknown-unknown\release\wasm_sample_bg.wasm