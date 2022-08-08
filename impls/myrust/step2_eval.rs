#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;

use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use std::ops::{Add, Sub, Mul, Div};
use std::collections::HashMap;
use types::*;

fn main() {
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        println!("{}", rep(&s));
    }
}

macro_rules! define_arithmetic_operations {
    ( $f:ident ) => (
        Rc::new(move |x: &[MalType]| {
            let a = x.get(0).and_then(|x| x.num());
            let b = x.get(1).and_then(|x| x.num());
            if a.is_none() || b.is_none() {
                Err(malerr!("Illegal args for the arithmetic operation."))
            } else {
                let (a, b) = (a.unwrap(), b.unwrap());
                Ok(MalType::Num(a.$f(b)))
            }
        })
    )
}

fn rep(s: &str) -> String {
    let mut repl_env: HashMap<String, MalFunc> = HashMap::new();
    let add = define_arithmetic_operations!(add);
    let sub = define_arithmetic_operations!(sub);
    let mul = define_arithmetic_operations!(mul);
    let div = define_arithmetic_operations!(div);
    repl_env.insert("+".to_string(), MalFunc::new("+", add));
    repl_env.insert("-".to_string(), MalFunc::new("-", sub));
    repl_env.insert("*".to_string(), MalFunc::new("*", mul));
    repl_env.insert("/".to_string(), MalFunc::new("/", div));

    READ(s).and_then(|x| EVAL(&x, &repl_env)).and_then(|x| PRINT(&x))
        .unwrap_or_else(|msg| { eprintln!("{}", msg); "".to_string() })
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

fn EVAL(ast: &MalType, repl_env: &HashMap<String, MalFunc>) -> Result<MalType> {
    eval_ast(ast, repl_env)
}

fn PRINT(ast: &MalType) -> Result<String> {
    Ok(printer::pr_str(ast))
}

/* step2_eval */

fn eval_ast(ast: &MalType, repl_env: &HashMap<String, MalFunc>) -> Result<MalType> {
    match ast {
        MalType::List(v) if v.is_empty() => Ok(ast.clone()),
        MalType::List(v) => {
            let maltypes = v.into_iter().map(|x| eval_ast(x, repl_env)).collect::<Result<Vec<_>>>()?;
            match maltypes.first().cloned() {
                Some(MalType::Symbol(s)) => {
                    let f = repl_env.get(&*s)
                        .ok_or_else(|| malerr!("Symbol \"{}\" is not defined.", s)).cloned()?.f;
                    f(&maltypes[1..])  // length of args can be 0
                },
                _ => Err(malerr!("Cannot eval not a symbol.")),
            }
        },
        MalType::Vec(v) => {
            let v: Result<Vec<_>> = v.into_iter().map(|x|
                EVAL(x, repl_env)).collect();
            Ok(MalType::Vec(v?))
        },
        MalType::HashMap(v) => {
            let mut v1: Vec<(MalType, MalType)> = Vec::new();
            for (x, y) in v {
                v1.push((EVAL(x, repl_env)?, EVAL(y, repl_env)?));
            }
            Ok(MalType::HashMap(v1))
        },
        _ => Ok(ast.clone()),
    }
}
