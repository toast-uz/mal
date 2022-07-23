#![allow(non_snake_case)]
use std::io::{stdin, stdout, Write};

fn main() {
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        println!("{}", rep(&s));
    }
}

fn rep(s: &str) -> &str {
    PRINT(EVAL(READ(s)))
}

fn READ(s: &str) -> &str {
    s
}

fn EVAL(s: &str) -> &str {
    s
}

fn PRINT(s: &str) -> &str {
    s
}