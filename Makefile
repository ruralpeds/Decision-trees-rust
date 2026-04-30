.PHONY: help build test run docker-up docker-down clean fmt clippy

help:
	@echo "cds-tree-rs development tasks"
	@echo "=============================="
	@echo ""
	@echo "Build & Test:"
	@echo "  make build          - Build all crates (debug)"
	@echo "  make build-release  - Build all crates (release)"
	@echo "  make test           - Run all tests"
	@echo "  make test-core      - Run cds-tree-core tests only"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt            - Format code with rustfmt"
	@echo "  make clippy         - Lint code with clippy"
	@echo "  make audit          - Security audit"
	@echo ""
	@echo "Development:"
	@echo "  make run            - Run API server (requires PostgreSQL)"
	@echo "  make docker-up      - Start Docker Compose services"
	@echo "  make docker-down    - Stop Docker Compose services"
	@echo "  make docker-logs    - View Docker Compose logs"
	@echo ""
	@echo "Database:"
	@echo "  make db-migrate     - Run database migrations"
	@echo "  make db-reset       - Reset database to initial state"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make coverage       - Generate test coverage report"

build:
	cargo build

build-release:
	cargo build --release

test:
	cargo test

test-core:
	cargo test -p cds-tree-core --lib

run:
	cargo run -p cds-tree-api

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy -- -D warnings

audit:
	cargo audit

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

docker-ps:
	docker-compose ps

db-migrate:
	sqlx migrate run --database-url "postgres://cds_user:cds_password@localhost:5432/cds_tree"

db-reset:
	docker-compose down -v
	docker-compose up -d postgres
	sleep 5
	sqlx migrate run --database-url "postgres://cds_user:cds_password@localhost:5432/cds_tree"

clean:
	cargo clean
	rm -rf target/

coverage:
	cargo tarpaulin --out Html

check-all: fmt-check clippy test
	@echo "✅ All checks passed!"

dev-setup: docker-up
	@sleep 5
	@echo "PostgreSQL is starting (check 'docker-compose ps' to verify)"
	@echo "Once PostgreSQL is ready, run:"
	@echo "  make db-migrate"
	@echo "  make run"
