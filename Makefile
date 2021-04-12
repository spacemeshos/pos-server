COLOR ?= auto # Valid COLOR options: {always, auto, never}
CARGO = RUSTFLAGS="-L ./crates/gpu-post/resources" cargo --color $(COLOR)

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
	@$(CARGO) test -- --test-threads=1

update:
	@$(CARGO) update
