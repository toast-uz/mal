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
        Ok(maltype) => PRINT(&EVAL(&maltype)),
        Err(err) => {
            eprintln!("{}", err);
            "".to_string()
        }
    }
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

fn EVAL(maltypes: &MalType) -> MalType {
    maltypes.clone()
}

fn PRINT(maltype: &MalType) -> String {
    printer::pr_str(maltype)
}