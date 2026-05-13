help:
	# Shows any commands below with ## after the make target name as a list of available commands
	@echo "Available Commands:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

dependencies: ## Installs all optional cargo tools
	cargo install cargo-edit
	cargo install cargo-udeps
	cargo install samply

build: ## Builds a new debug build
	cargo build
	npm run build

build-release: ## Builds a new release build
	cargo build --release
	npm run build

build-profiling: ## Builds a new profiling build
	cargo build --profiling
	npm run build

test: ## Runs all test suites
	cargo test --all

fmt: ## Formats the source code
	cargo fmt --all
	npm run format

lint: ## Checks for any syntactic sugar
	cargo clippy --all-features --all --tests --examples -- -D clippy::all -D warnings
	npm run lint

upgrade: ## Upgrades all project dependencies
	cargo upgrade --verbose

udeps: ## Find unused dependencies
	cargo +nightly udeps

clean: ## Cleans any intermediate build data
	cargo clean
	npm run clean