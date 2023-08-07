echo "BUILDING"
cargo build --target wasm32-unknown-unknown --no-default-features --features web --release
echo "PACKAKING"
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/foddian_jam.wasm --no-typescript
echo "OPTIMIZING"
wasm-opt -O4 -o ./out/foddian_jam_bg.wasm  ./out/foddian_jam_bg.wasm
echo "COMPRESSING"
7z a web.zip out