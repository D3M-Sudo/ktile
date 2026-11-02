CARGO ?= cargo
PYTHON ?= python3
BASH ?= bash

.PHONY: fmt test clippy build license-check x11-integration check clean

fmt:
	$(CARGO) fmt --all -- --check

test:
	$(CARGO) test --all-features --locked

clippy:
	$(CARGO) clippy --all-targets --all-features --locked -- -D warnings

build:
	$(CARGO) build --release --all-features --locked

license-check:
	$(PYTHON) scripts/check_dependency_licenses.py

# Requires Xvfb, Openbox, x11-utils and xdotool; runs only on a disposable X server.
x11-integration: build
	$(BASH) scripts/test-x11-integration.sh

check: fmt test clippy license-check build

clean:
	$(CARGO) clean
