.PHONY: fmt lint build run-black run-white run-world up down logs

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --workspace -- -D warnings || true

build:
	cargo build --workspace

run-black:
	cargo run -p kernel-black

run-white:
	cargo run -p kernel-white

run-world:
	cargo run -p world-noise

up:
	docker compose up --build -d

down:
	docker compose down

logs:
	docker compose logs -f --tail=200
