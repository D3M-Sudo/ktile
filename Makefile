CARGO ?= cargo

.PHONY: fmt test clippy build check clean

fmt:
	$(CARGO) fmt --all -- --check

test:
	$(CARGO) test --all-features --locked

clippy:
	$(CARGO) clippy --all-targets --all-features --locked -- -D warnings

build:
	$(CARGO) build --release --all-features --locked

check: fmt test clippy build

clean:
	$(CARGO) clean
