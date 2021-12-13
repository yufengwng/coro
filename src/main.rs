use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

use coro::vm::CoRes;
use coro::vm::CoVM;

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

    let status = if args.len() > 1 {
        run_file(&args[1])
    } else {
        run_repl()
    };

    process::exit(status);
}

fn run_file(path: &str) -> i32 {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[coro] error while reading file: {}", path);
            eprintln!("[coro] {}", e);
            return STATUS_GENERAL_ERR;
        }
    };
    match CoVM::eval(&src) {
        CoRes::Ok => STATUS_OK,
        CoRes::CompileErr => STATUS_COMPILE_ERR,
        CoRes::RuntimeErr => STATUS_RUNTIME_ERR,
    }
}

fn run_repl() -> i32 {
    let mut comain = CoVM::build("").unwrap();
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

        let def = match CoVM::compile(&src) {
            Ok(rc) => rc,
            Err(e) => {
                eprintln!("[coro] compile error:\n{}", e);
                continue;
            }
        };

        CoVM::rewind(&mut comain, def);
        let val = match CoVM::run(&mut comain) {
            Ok(val) => val,
            Err(msg) => {
                eprintln!("[coro] runtime error: {}", msg);
                continue;
            }
        };

        if cfg!(feature = "dbg") {
            eprintln!("{}", comain);
            eprintln!("[coro] value: {}", val);
        }
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
