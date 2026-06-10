help:
	# Shows any commands below with ## after the make target name as a list of available commands
	@echo "Available Commands:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

dependencies: ## Installs all optional cargo tools
	npm install
	cargo install cargo-edit

build: ## Builds a new debug build
	cargo build
	npm run build

build-release: ## Builds a new release build
	cargo build --release
	npm run build

build-docker: ## Builds a release docker build
	docker build --tag roniai/frontend:local --file ./deploy/docker/frontend.dockerfile .

test: ## Runs all test suites
	mkdir -p ./backend/target
	cargo test --all -- --test-threads=1 --nocapture

fmt: ## Formats the source code
	cargo fmt --all
	npm run format

lint: ## Checks for any syntactic sugar
	cargo clippy --all-features --all --tests --examples -- -D clippy::all -D warnings
	npm run lint

upgrade: ## Upgrades all project dependencies
	cargo upgrade --verbose
	npm update --save

clean: ## Cleans any intermediate build data
	cargo clean
	npm run clean