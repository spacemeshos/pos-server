COLOR ?= auto # Valid COLOR options: {always, auto, never}
CARGO = RUSTFLAGS="-L ./crates/pos-compute/resources" cargo --color $(COLOR)
CARGO_TEST =  RUSTFLAGS="-L ./crates/pos-compute/resources" cargo test --test server_tests test_api --all-features --manifest-path ./crates/pos-service/Cargo.toml -- --nocapture


.PHONY: all bench build check clean doc install publish run test update format

all: build

format:
	@$(CARGO) fmt

bench:
	@$(CARGO) bench

build: format
	 @$(CARGO) build --color=always --all --all-targets

check:
	@$(CARGO) check

clean:
	@$(CARGO) clean

doc:
	@$(CARGO) doc

install: build
	@$(CARGO) install

publish:
	@$(CARGO) publish

run: build
	@$(CARGO) run -p pos-service

test: build
	@$(CARGO_TEST)

update:
	@$(CARGO) update
