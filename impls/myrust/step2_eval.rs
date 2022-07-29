#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;

use std::io::{stdin, stdout, Write};
use std::ops::{Add, Sub, Mul, Div};
use std::collections::HashMap;
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
    READ(s).and_then(|x| EVAL(&x)).and_then(|x| PRINT(&x))
        .unwrap_or_else(|msg| { eprintln!("{}", msg); "".to_string() })
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

fn EVAL(maltype: &MalType) -> Result<MalType> {
    eval_ast(maltype)
}

fn PRINT(maltype: &MalType) -> Result<String> {
    Ok(printer::pr_str(maltype))
}

/* step2_eval */

fn eval_ast(maltype: &MalType) -> Result<MalType> {
    match maltype {
        MalType::List(v) if v.is_empty() => Ok(maltype.clone()),
        MalType::List(v) => {
            let maltypes = v.into_iter().map(|x| eval_ast(x)).collect::<Result<Vec<_>>>()?;
            eval_func(&maltypes[0], &maltypes[1..].to_vec())
        },
        MalType::Vec(v) => {
            let maltypes: Result<Vec<_>> = v.into_iter().map(|x| eval_ast(x)).collect();
            Ok(MalType::Vec(maltypes?))
        },
        MalType::HashMap(hm) => {
            let mut hm_maltype: HashMap::<MalType, MalType> = HashMap::new();
            for (k, v) in hm {
                hm_maltype.insert(k.clone(), eval_ast(v)?);
            }
            Ok(MalType::HashMap(hm_maltype))
        },
        _ => Ok(maltype.clone()),
    }
}

fn eval_func(func: &MalType, args: &Vec<MalType>) -> Result<MalType> {
    if let MalType::Symbol(s) = func {
        match s as &str {
            _ if args.len() != 2 => Err(malerr!("Illegal number: {} of args.", args.len())),
            "+" => Ok(args[0].clone() + args[1].clone()),
            "-" => Ok(args[0].clone() - args[1].clone()),
            "*" => Ok(args[0].clone() * args[1].clone()),
            "/" => Ok(args[0].clone() / args[1].clone()),
            _ => Err(malerr!("Unknown Symbol: \"{}\"", func)),
        }
    } else {
        Err(malerr!("List must begin a Symbol, but found {}", func))
    }
}

macro_rules! malfunc {
    ( $t:ident, $f:ident ) => (
        impl $t for MalType {
            type Output = Self;
            fn $f(self, other: Self) -> Self::Output {
                match (self, other) {
                    (MalType::Int(x), MalType::Int(y)) => MalType::Int(x.$f(&y)),
                    (MalType::Int(x), MalType::Float(y)) => MalType::Float((x as f64).$f(&y)),
                    (MalType::Float(x), MalType::Int(y)) => MalType::Float(x.$f(&(y as f64))),
                    (MalType::Float(x), MalType::Float(y)) => MalType::Float(x.$f(&y)),
                    (x, y) => panic!("Cannot calc between {} and {}", x, y),
                }
            }
        }
    )
}

malfunc!(Add, add);
malfunc!(Sub, sub);
malfunc!(Mul, mul);
malfunc!(Div, div);

