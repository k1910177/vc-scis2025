#!/usr/bin/make -f

MAKEFLAGS+=--no-print-directory

# Default target is build
default: build

# Define variables
CARGO=cargo
FORGE=forge
PY=python
PIP=pip
CONTRACTS_PATH=./contracts
BINDINGS_FOLDER=bindings
BINDINGS_PATH=$(CONTRACTS_PATH)/$(BINDINGS_FOLDER)

# Target for generating bindings
bind:
# Generate new bindings
	@$(FORGE) clean --root $(CONTRACTS_PATH)
	@rm -rf $(BINDINGS_PATH)
	@$(FORGE) bind --bindings-path $(BINDINGS_PATH) \
	--root $(CONTRACTS_PATH) --alloy --skip-cargo-toml --overwrite

# Target for building the project
build: 
	@$(MAKE) bind
	@$(CARGO) build

# Target for building the project in release mode
build-release: 
	@$(MAKE) bind
	@$(CARGO) build --release

# Target for cleaning the project
clean:
	@$(FORGE) clean --root $(CONTRACTS_PATH)
	@$(CARGO) clean

# Target for formatting the code
fmt:
	@$(FORGE) fmt --check --root $(CONTRACTS_PATH)
	@$(CARGO) fmt

# Target for running tests
test:
	@$(FORGE) test --root $(CONTRACTS_PATH)
	@$(CARGO) test

# Target for installing forge dependencies
setup:
	@$(FORGE) install --root $(CONTRACTS_PATH)
	@$(PY) -m venv .venv
	@source .venv/bin/activate
	@$(PIP) install -r requirements.txt

# Declare phony targets
.PHONY: bind build build-release clean fmt bindings
