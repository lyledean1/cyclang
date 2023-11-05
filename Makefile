default: install

all: hooks install build

run:
	./bin/main

#run clang on the llvm ir to generate a binary 
build-ir:
	clang ./bin/main.ll -o ./bin/main

install-locally:
	cargo install --path=./cyclang

test: 
	cargo test -- --test-threads=1

test-release:
	cargo test --release -- --test-threads=1

test-parser:
	cargo test -- parser

clean:
	rm -rf ./bin/main*

h help:
	@grep '^[a-z]' Makefile

.PHONY: hooks
hooks:
	cd .git/hooks && ln -s -f ../../hooks/pre-push pre-push

install-mdbook:
	cargo install mdbook && cargo install mdbook-mermaid


s serve:
	cd book && mdbook serve


build-book:
	cd book && mdbook build

cargo-publish:
	cargo publish cyclang-macros && cargo publish cyclang