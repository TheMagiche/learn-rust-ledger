.PHONY: gen dev wasm build test lint

gen:
	cargo run -p gen-sources --quiet

dev: gen
	./scripts/dev.sh

wasm: gen
	wasm-pack build crates/wasm --target web --out-dir ../../web/src/wasm

build: wasm
	npm --prefix web install
	npm --prefix web run build
	cargo build --release -p spend

test: wasm
	cargo test --workspace
	npm --prefix web install
	npm --prefix web test

lint: wasm
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings
	npm --prefix web install
	npm --prefix web run lint
	npm --prefix web run typecheck
