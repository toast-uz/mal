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
    let mut repl_env: Env = Env::new(None);
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        println!("{}", rep(&s, &mut repl_env));
    }
}

fn rep<'a>(s: &str, repl_env: &mut Env) -> String {
    repl_env.set("+", &MalType::Fn(MalFunc::new("+", Rc::new(add))));
    repl_env.set("-", &MalType::Fn(MalFunc::new("-", Rc::new(sub))));
    repl_env.set("*", &MalType::Fn(MalFunc::new("*", Rc::new(mul))));
    repl_env.set("/", &MalType::Fn(MalFunc::new("/", Rc::new(div))));

    READ(s).and_then(|x| EVAL(&x, repl_env)).and_then(|x| PRINT(&x))
        .unwrap_or_else(|msg| { eprintln!("Err: {}", msg); "".to_string() })
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

fn EVAL<'a>(maltype: &MalType, repl_env: &mut Env<'a>) -> Result<MalType> {
    if let MalType::List(v) = maltype.clone() {
        if let Some(MalType::Symbol(s)) = v.first() {
            if s == "def!" {
                if v.len() == 3 { if let MalType::Symbol(t) = &v[1].clone() {
                    let value = EVAL(&v[2].clone(), repl_env)?;
                    repl_env.set(t, &value);
                    return EVAL(&v[1].clone(), repl_env);
                } }
                return Err(malerr!("Syntax error of 'def!'."));
            } else if s == "let*" {
                let mut new_repl_env = Env::new(Some(repl_env));
                if v.len() == 3 { match v[1].clone() {
                    MalType::List(v1) | MalType::Vec(v1) if v1.len() % 2 == 0 => {
                        for x in v1.chunks(2) {
                            if let MalType::Symbol(t) = &x[0].clone() {
                                let value = EVAL(&x[1].clone(), &mut new_repl_env)?;
                                new_repl_env.set(t, &value);
                            } else {
                                return Err(malerr!("List of first arg for 'let*' must be Symbols and args."));
                            }
                        }
                        return EVAL(&v[2].clone(), &mut new_repl_env);
                    },
                    _ => { return Err(malerr!("The first arg of let* must be a List or Vec with even elems.")); },
                } }
                return Err(malerr!("Syntax error of 'let*'."));
            } else if s == "special" {
                return Err(malerr!("Special symbol is not implemented."));
            }
        }
    }
    eval_ast(maltype, repl_env)
}

fn PRINT(maltype: &MalType) -> Result<String> {
    Ok(printer::pr_str(maltype))
}

/* step2_eval */

fn eval_ast(maltype: &MalType, repl_env: &mut Env) -> Result<MalType> {
    match maltype {
        MalType::List(v) if v.is_empty() => Ok(maltype.clone()),
        MalType::List(v) => {
            let maltypes = v.into_iter().map(|x| EVAL(x, repl_env)).collect::<Result<Vec<_>>>()?;
            match maltypes.first().cloned() {
                Some(MalType::Symbol(s)) => {
                    let symbol_content = repl_env.get(&*s)?;
                    match symbol_content {
                        MalType::Fn(f) => (f.f)(&maltypes[1..]), // length of args can be 0
                        _ => Err(malerr!("Symbol {} is not a function.", s)),
                    }
                }
                _ => Err(malerr!("Cannot eval not a symbol.")),
            }
        },
        MalType::Vec(v) => {
            let maltypes: Result<Vec<_>> = v.into_iter().map(|x| EVAL(x, repl_env)).collect();
            Ok(MalType::Vec(maltypes?))
        },
        MalType::HashMap(hm) => {
            let mut hm_maltype: HashMap::<MalType, MalType> = HashMap::new();
            for (k, v) in hm { hm_maltype.insert(k.clone(), EVAL(v, repl_env)?); }
            Ok(MalType::HashMap(hm_maltype))
        },
        MalType::Symbol(s) => {
            let symbol_content = repl_env.get(&*s)?;
            match symbol_content {
                MalType::Fn(_) => Ok(maltype.clone()),
                _ => EVAL(&symbol_content, repl_env),
            }
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

