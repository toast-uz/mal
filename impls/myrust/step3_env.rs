#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;
mod env;

use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use std::ops::{Add, Sub, Mul, Div};
use std::collections::HashMap;
use types::*;
use env::*;

 fn main() {
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        println!("{}", rep(&s));
    }
}

fn rep<'a>(s: &str) -> String {
    let mut repl_env: Env = Env::new(None);
    repl_env.set("+", &MalFunc::new("+", Rc::new(add)));
    repl_env.set("-", &MalFunc::new("-", Rc::new(sub)));
    repl_env.set("*", &MalFunc::new("*", Rc::new(mul)));
    repl_env.set("/", &MalFunc::new("/", Rc::new(div)));

    READ(s).and_then(|x| EVAL(&x, &repl_env)).and_then(|x| PRINT(&x))
        .unwrap_or_else(|msg| { eprintln!("{}", msg); "".to_string() })
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

fn EVAL(maltype: &MalType, repl_env: &Env) -> Result<MalType> {
    eval_ast(maltype, repl_env)
}

fn PRINT(maltype: &MalType) -> Result<String> {
    Ok(printer::pr_str(maltype))
}

/* step2_eval */

fn eval_ast(maltype: &MalType, repl_env: &Env) -> Result<MalType> {
    match maltype {
        MalType::List(v) if v.is_empty() => Ok(maltype.clone()),
        MalType::List(v) => {
            let maltypes = v.into_iter().map(|x| eval_ast(x, repl_env)).collect::<Result<Vec<_>>>()?;
            match maltypes.first().cloned() {
                Some(MalType::Symbol(s)) => {
                    (repl_env.get(&*s)?.f)(&maltypes[1..])  // length of args can be 0
                },
                _ => Err(malerr!("Cannot eval not a symbol.")),
            }
        },
        MalType::Vec(v) => {
            let maltypes: Result<Vec<_>> = v.into_iter().map(|x| eval_ast(x, repl_env)).collect();
            Ok(MalType::Vec(maltypes?))
        },
        MalType::HashMap(hm) => {
            let mut hm_maltype: HashMap::<MalType, MalType> = HashMap::new();
            for (k, v) in hm { hm_maltype.insert(k.clone(), eval_ast(v, repl_env)?); }
            Ok(MalType::HashMap(hm_maltype))
        },
        _ => Ok(maltype.clone()),
    }
}

macro_rules! malfunc_binomial_number {
    ( $s:expr, $f:ident ) => (
        fn $f(v: &[MalType]) -> Result<MalType> {
            if v.len() != 2 { return Err(malerr!(
                "Illegal number of args: {} for the binomial operator \"{}\".", v.len(), $s)
            ); }
            let (x, y) = (v[0].clone(), v[1].clone());
            match (x, y) {
                (MalType::Int(x), MalType::Int(y)) => Ok(MalType::Int(x.$f(&y))),
                (MalType::Int(x), MalType::Float(y)) => Ok(MalType::Float((x as f64).$f(&y))),
                (MalType::Float(x), MalType::Int(y)) => Ok(MalType::Float(x.$f(&(y as f64)))),
                (MalType::Float(x), MalType::Float(y)) => Ok(MalType::Float(x.$f(&y))),
                (x, y) => Err(malerr!("Cannot calc \"{}\" between {} and {}", $s, x, y)),
            }
        }
    )
}

malfunc_binomial_number!("+", add);
malfunc_binomial_number!("-", sub);
malfunc_binomial_number!("*", mul);
malfunc_binomial_number!("/", div);
