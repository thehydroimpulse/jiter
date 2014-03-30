test:
	mkdir -p target
	rustc --test --out-dir target src/lib.rs
	./target/lib

bin:
	mkdir -p target
	rustc --out-dir target src/bin.rs
	./target/bin
