#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;
mod env;

use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use std::ops::{Add, Sub, Mul, Div};
use types::*;
use env::*;

macro_rules! define_arithmetic_operations {
    ( $f:ident ) => (
        Rc::new(move |x: &[MalType]| {
            let a = x.get(0).and_then(|x| x.num());
            let b = x.get(1).and_then(|x| x.num());
            if a.is_none() || b.is_none() {
                Err(malerr!("Illegal args for the arithmetic operation."))
            } else {
                let (a, b) = (a.unwrap(), b.unwrap());
                Ok(MalType::from(a.$f(b)))
            }
        })
    )
}

 fn main() {
    let mut repl_env: Env = Env::new(None);
    let add = define_arithmetic_operations!(add);
    let sub = define_arithmetic_operations!(sub);
    let mul = define_arithmetic_operations!(mul);
    let div = define_arithmetic_operations!(div);
    repl_env.set("+", &MalType::Fn(MalFunc::new("+", add)));
    repl_env.set("-", &MalType::Fn(MalFunc::new("-", sub)));
    repl_env.set("*", &MalType::Fn(MalFunc::new("*", mul)));
    repl_env.set("/", &MalType::Fn(MalFunc::new("/", div)));
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        println!("{}", rep(&s, &mut repl_env));
    }
}

fn rep<'a>(s: &str, repl_env: &mut Env) -> String {
    READ(s).and_then(|x| EVAL(&x, repl_env)).and_then(|x| PRINT(&x))
        .unwrap_or_else(|msg| { eprintln!("Err: {}", msg); "".to_string() })
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

fn EVAL<'a>(ast: &MalType, repl_env: &mut Env) -> Result<MalType> {
    match ast.get(0).and_then(|x| x.symbol()).as_deref() {
        Some("def!") => {
            let key = ast.get(1).and_then(|x| x.symbol());
            let value = ast.get(2).clone();
            if key.is_none() || value.is_none() {
                return Err(malerr!("Syntax error of 'def!'."));
            }
            let (key, value) = (key.unwrap(), EVAL(&value.unwrap(), repl_env)?);
            repl_env.set(&key, &value);
            EVAL(&MalType::Symbol(key), repl_env)
        },
        Some("let*") => {
            let mut temp_env = Env::new(Some(repl_env));
            let keys = ast.get(1)
                .and_then(|x| x.list_or_vec())
                .filter(|v| v.len() % 2 == 0);
            let value = ast.get(2).clone();
            if keys.is_none() || value.is_none() {
                return Err(malerr!("Syntax error of 'let*'."));
            }
            let (keys, value) = (keys.unwrap(), value.unwrap());
            for x in keys.chunks(2) {
                if let MalType::Symbol(s) = &x[0].clone() {
                    let v = EVAL(&x[1].clone(), &mut temp_env)?;
                    temp_env.set(s, &v);
                } else {
                    return Err(malerr!("List of first arg for 'let*' or 'fn*' must be Symbols and args."));
                }
            }
            EVAL(&value, &mut temp_env)
        },
        _ => eval_ast(ast, repl_env),
    }
}

fn PRINT(ast: &MalType) -> Result<String> {
    Ok(printer::pr_str(ast))
}

/* step2_eval */

fn eval_ast(ast: &MalType, repl_env: &mut Env) -> Result<MalType> {
    match ast {
        MalType::List(v) if v.is_empty() => Ok(ast.clone()),
        MalType::List(v) => {
            let v = v.into_iter().map(|x| EVAL(x, repl_env)).collect::<Result<Vec<_>>>()?;
            match v.first().cloned() {
                Some(MalType::Symbol(s)) => {
                    let symbol_content = repl_env.get(&*s)?;
                    match symbol_content {
                        MalType::Fn(f) => (f.f)(&v[1..]), // length of args can be 0
                        _ => Err(malerr!("Symbol {} is not a function.", s)),
                    }
                }
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
        MalType::Symbol(s) => {
            let symbol_content = repl_env.get(&*s)?;
            match symbol_content {
                MalType::Fn(_) => Ok(ast.clone()),
                _ => EVAL(&symbol_content, repl_env),
            }
        },
        _ => Ok(ast.clone()),
    }
}
