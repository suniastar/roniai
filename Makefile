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

build: ## Builds a new release build
	cargo build --release

build: ## Builds a new profiling build
	cargo build --profiling

test: ## Runs all test suites
	cargo test-all

fmt: ## Formats the source code
	cargo fmt-all

lint: ## Checks for any syntactic sugar
	cargo lint-all

upgrade: ## Upgrades all project dependencies
	cargo upgrade --verbose

udeps: ## Find unused dependencies
	cargo +nightly udeps

clean: ## Cleans any intermediate build data
	cargo clean
