WASM_TARGET := wasm32-unknown-unknown
WASM := target/$(WASM_TARGET)/release/streampay_contract.wasm

.PHONY: build
build:
	cargo build --target $(WASM_TARGET) --release

.PHONY: test
test:
	cargo test
