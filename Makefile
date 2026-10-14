.PHONY: gen dev build test lint

gen:
	cargo run -p gen-sources --quiet

dev: gen
	./scripts/dev.sh

build: gen
	npm --prefix web install
	npm --prefix web run build
	cargo build --release -p spend

test: gen
	cargo test --workspace
	npm --prefix web install
	npm --prefix web test

lint: gen
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings
	npm --prefix web install
	npm --prefix web run lint
	npm --prefix web run typecheck
