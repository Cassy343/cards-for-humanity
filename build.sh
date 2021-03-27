wasm-pack build ./client --target web --out-dir ../target/client-out

if [ $# -eq 0 ]
then
    cargo build
else
    cargo build "$@"
fi
