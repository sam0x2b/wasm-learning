$p = gcim win32_process | where {$_.commandline -clike "*python.exe*server.py*"} | select ProcessId -expandproperty ProcessId
if ($null -ne $p) { kill $p } # cant `-erroraction Ignore`
del target\wasm32-unknown-unknown\debug\wasm_sample.js -erroraction Ignore
del target\wasm32-unknown-unknown\debug\wasm_sample_bg.wasm -erroraction Ignore
cargo build --target wasm32-unknown-unknown
cd target\wasm32-unknown-unknown/debug
wasm-bindgen --target web --no-typescript --out-dir . wasm_sample.wasm
wasm-gc wasm_sample_bg.wasm
cd ..\..\..\www
copy ..\target\wasm32-unknown-unknown\debug\wasm_sample.js .
copy ..\target\wasm32-unknown-unknown\debug\wasm_sample_bg.wasm .
cd ..
start python.exe "server.py" # -NoNewWindow
