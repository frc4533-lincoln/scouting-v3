.POSIX:

all: wasm server

wasm:
	cargo b -r --target wasm32-unknown-unknown -p scouting-v3-wasm #--bin scouting-v3
	wasm-bindgen --target web --no-typescript --out-dir dist target/wasm32-unknown-unknown/release/scouting-v3.wasm
	cp assets/* dist

server:
	cargo b -r -p scouting-v3-server

.PHONY: all wasm server
