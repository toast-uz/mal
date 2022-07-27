#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;

use std::io::{stdin, stdout, Write};
use types::*;

type Result<T> = std::result::Result<T, MalError>;

fn main() {
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        println!("{}", rep(&s));
    }
}

fn rep(s: &str) -> String {
    match READ(s) {
        Ok(mal_type) => PRINT(&EVAL(&mal_type)),
        Err(err) => {
            eprintln!("{}", err);
            "".to_string()
        }
    }
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

fn EVAL(mal_types: &MalType) -> MalType {
    mal_types.clone()
}

fn PRINT(mal_type: &MalType) -> String {
    printer::pr_str(mal_type)
}