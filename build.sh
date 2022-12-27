cargo build --target wasm32-unknown-unknown
cd target/wasm32-unknown-unknown/debug
wasm-bindgen --target web --no-typescript --out-dir . wasm_sample.wasm
wasm-gc wasm_sample_bg.wasm
cd ../../../www
cp ../target/wasm32-unknown-unknown/debug/wasm_sample.js .
cp ../target/wasm32-unknown-unknown/debug/wasm_sample_bg.wasm .

