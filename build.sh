#!/usr/bin/env sh
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target nodejs

cp ./typescript/* ./pkg
rm -rf ./pkg/package.json ./pkg/wasm_validator.d.ts