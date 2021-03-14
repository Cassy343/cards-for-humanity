cargo build --package common
wasm-pack build client/ --target web --out-dir ../target/client-out
cargo build --package server