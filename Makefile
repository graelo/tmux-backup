.PHONY: all clean test

release:
	RUSTFLAGS="-Ctarget-cpu=native" cargo build --release
