# rlox — Makefile
#
# Common workflows:
#   make                 # show help
#   make release run-demo
#   make test-official   # full Crafting Interpreters jlox suite
#   make test-official FILTER=inheritance

.DEFAULT_GOAL := help

ROOT        := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
RLOX_DEBUG  := $(ROOT)target/debug/rlox
RLOX        := $(ROOT)target/release/rlox
CRAFTING    ?= $(HOME)/Projects/craftinginterpreters
DART        ?= dart
CARGO       ?= cargo

.PHONY: help build release check clippy fmt fmt-check test clean \
        run run-file run-release demo repl \
        test-official test-official-quick generate-ast

help: ## Show this help
	@printf "Usage: make [target] [VAR=value]\n\nTargets:\n"
	@grep -E '^[a-zA-Z0-9_.-]+:.*##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*## "}; {printf "  %-22s %s\n", $$1, $$2}'
	@printf "\nVariables:\n"
	@printf "  FILE=%s\n" "$(FILE)"
	@printf "  FILTER=%s\n" "$(FILTER)"
	@printf "  CRAFTING=%s\n" "$(CRAFTING)"
	@printf "\nExamples:\n"
	@printf "  make run-file FILE=tests/tree_walk_interpreter.rlox\n"
	@printf "  make test-official FILTER=assignment\n"
	@printf "  make test-official CRAFTING=../craftinginterpreters\n"

build: ## Build debug binary
	$(CARGO) build

release: ## Build optimized binary (required for test-official)
	$(CARGO) build --release

check: ## Type-check without producing a binary
	$(CARGO) check

clippy: ## Run Clippy lints
	$(CARGO) clippy --all-targets -- -D warnings

fmt: ## Format Rust sources
	$(CARGO) fmt

fmt-check: ## Verify formatting without writing files
	$(CARGO) fmt --check

test: ## Run Rust unit tests
	$(CARGO) test

clean: ## Remove build artifacts
	$(CARGO) clean
	@rm -f generate_ast

run: build ## Run the REPL (debug build)
	$(CARGO) run

repl: run ## Alias for run

run-file: release ## Run a script: make run-file FILE=tests/foo.rlox
ifndef FILE
	$(error FILE is required, e.g. make run-file FILE=tests/tree_walk_interpreter.rlox)
endif
	$(RLOX) $(FILE)

run-release: release ## Run the REPL (release build)
	$(RLOX)

demo: release ## Run the Part II summary program
	$(RLOX) tests/tree_walk_interpreter.rlox

test-official: release ## Run the official Crafting Interpreters jlox suite
	@test -x "$(RLOX)" || (echo "missing $(RLOX); run 'make release' first" && exit 1)
	@test -f "$(CRAFTING)/tool/bin/test.dart" || \
		(echo "missing $(CRAFTING)/tool/bin/test.dart; set CRAFTING=..." && exit 1)
	cd "$(CRAFTING)" && $(DART) tool/bin/test.dart jlox \
		$(FILTER) \
		--interpreter "$(abspath $(RLOX))"

test-official-quick: release ## Smoke-test one official case
	$(MAKE) test-official FILTER=assignment/global.lox

generate-ast: ## Regenerate expr AST boilerplate (book tool)
	rustc tools/generate_ast.rs -o generate_ast
	./generate_ast src/expr
