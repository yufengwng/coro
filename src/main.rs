use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

use coro::vm;

const STATUS_OK: i32 = 0;
const STATUS_COMPILE_ERR: i32 = 1;
const STATUS_RUNTIME_ERR: i32 = 2;
const STATUS_GENERAL_ERR: i32 = 3;
const STATUS_USAGE_ERR: i32 = 4;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("usage: coro [script]");
        process::exit(STATUS_USAGE_ERR);
    }

    let mut vm = vm::CoVM;
    let status = if args.len() > 1 {
        run_file(&mut vm, &args[1])
    } else {
        run_repl(&mut vm)
    };

    process::exit(status);
}

fn run_file(vm: &mut vm::CoVM, path: &str) -> i32 {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[coro] error while reading file: {}", path);
            eprintln!("[coro] {}", e);
            return STATUS_GENERAL_ERR;
        }
    };
    match vm.run(&src) {
        vm::CoRes::Ok => STATUS_OK,
        vm::CoRes::CompileErr => STATUS_COMPILE_ERR,
        vm::CoRes::RuntimeErr => STATUS_RUNTIME_ERR,
    }
}

fn run_repl(vm: &mut vm::CoVM) -> i32 {
    println!("[coro-lang]");
    loop {
        let src = match repl_read() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[coro] {}", e);
                return STATUS_GENERAL_ERR;
            }
        };
        if src.is_empty() {
            continue;
        }
        vm.run(&src);
    }
}

fn repl_read() -> io::Result<String> {
    print!("> ");
    io::stdout().flush()?;

    let mut lines = Vec::new();
    loop {
        let mut buffer = String::new();
        let len = io::stdin().read_line(&mut buffer)?;
        if len == 0 {
            // Reached EOF, erase control char by writing backspace and exit.
            println!("{}", 8_u8 as char);
            process::exit(STATUS_OK);
        }

        let input = buffer.trim();
        if input.ends_with(";;") {
            let line = input.strip_suffix(";;").unwrap();
            lines.push(line.to_owned());
            break;
        } else {
            lines.push(input.to_owned());
        }

        print!("· ");
        io::stdout().flush()?;
    }

    let src = lines.join("\n").trim().to_owned();
    Ok(src)
}
