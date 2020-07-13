.PHONY: build-frontend build-backend clean

run: build-backend build-frontend
	cargo run

build: build-backend build-frontend

build-frontend:
	cd frontend && wasm-pack build --no-typescript --target web --out-name main --out-dir ./static

build-backend:
	cargo build
