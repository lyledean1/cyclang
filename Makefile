default: install

all: hooks install build

run:
	./bin/main

build-stdlib:
	cd crates/codegen/src/stdlib && clang -c -emit-llvm -O0 types.c -o types.bc

build-stdlib-ir:
	cd crates/codegen/src/stdlib && clang -S -emit-llvm -O0 types.c -o types.ll

#run clang on the llvm ir to generate a binary 
build-ir:
	clang ./bin/main.ll -o ./bin/main

install-local: build-stdlib
	cargo install --path=./crates/cyclang

test-local: 
	cargo test -- --test-threads=1

test-local-release:
	cargo test --release -- --test-threads=1

test-local-parser:
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

build-ubuntu-docker:
	cd .devcontainer/ubuntu-x86_64 && docker build -t cyclang-base .

set-x86-64-env:
	echo 'source $$HOME/.cargo/env' >> $$HOME/.bashrc

test-x86-64-docker: build-ubuntu-docker
	docker run -it -v "${PWD}:/cyclang" cyclang-base make test-local

fib-wasm:
	cargo run -- --file=./examples/wasm/fib.cyc --target=wasm --emit-llvm-ir
	@echo "Generating WASM file from LLVM IR"
	@opt -O3 -S ./bin/main.ll -o ./bin/opt.ll
	@llc -march=wasm32 -filetype=obj ./bin/main.ll -o ./bin/fib.o
	@wasm-ld --no-entry --export-all -o ./bin/fib.wasm ./bin/fib.o
	@llc -march=wasm32 -filetype=obj ./bin/opt.ll -o ./bin/fib-opt.o
	@wasm-ld --no-entry --export-all -o ./bin/fib-opt.wasm ./bin/fib-opt.o
	@cp ./bin/fib.wasm ./examples/wasm/fib.wasm
	@cp ./bin/fib-opt.wasm ./examples/wasm/fib-opt.wasm
	@echo "---------------------------------"
	node ./examples/wasm/fib.js

cargo-publish:
	cargo publish -p cyclang-macros && cargo publish -p cyclang
