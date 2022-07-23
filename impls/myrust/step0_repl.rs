#![allow(non_snake_case)]
use std::io::{stdin, stdout, Write};

fn main() {
    loop {
        let mut s = String::new();
        print!("user> ");
        stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        println!("{}", rep(&s));
    }
}

fn rep<'a>(s: &'a str) -> &'a str {
    PRINT(EVAL(READ(s)))
}

fn READ<'a>(s: &'a str) -> &'a str {
    s
}

fn EVAL<'a>(s: &'a str) -> &'a str {
    s
}

fn PRINT<'a>(s: &'a str) -> &'a str {
    s
}