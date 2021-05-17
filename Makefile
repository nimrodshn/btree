
.PHONY: build
build:
	cargo build --verbose

.PHONY:	test
test:
	cargo test --verbose -- --test-threads=1 --nocapture