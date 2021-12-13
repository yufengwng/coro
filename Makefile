build:
	cargo build --features=ast,dbg,instr

run:
	cargo run --features=ast,dbg,instr

release:
	cargo build --release

clean:
	cargo clean
