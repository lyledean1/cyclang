run:
	./bin/main

#run clang on the llvm ir to generate a binary 
build-ir:
	clang ./bin/main.ll -o ./bin/main

test:
	cargo test -- --test-threads=1

clean:
	rm -rf ./bin/main*