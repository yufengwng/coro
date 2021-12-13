build:
	cargo build --features=ast,dbg,instr,stack

run:
	cargo run --features=ast,dbg,instr,stack

release:
	cargo build --release

clean:
	cargo clean
