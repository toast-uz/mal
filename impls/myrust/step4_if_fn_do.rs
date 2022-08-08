#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;
mod env;
mod core;

use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use itertools::Itertools;
use types::*;
use env::*;

const DEBUG: bool = false;
macro_rules! dbg {( $( $x:expr ),* ) => ( if DEBUG {eprintln!($( $x ),* )})}

 fn main() {
    let mut env: Env = Env::new(None);
    core::ns().iter().for_each(|f|
        { env.set(&f.name, &MalType::Fn(f.clone())); });
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        if !s.trim().is_empty() {
            println!("{}", rep(&s, &mut env));
            dbg!(" env: {}", env);
        }
    }
}

fn rep(s: &str, env: &mut Env) -> String {
    READ(s).and_then(|x| EVAL(&x, env)).and_then(|x| PRINT(&x))
        .unwrap_or_else(|msg| { eprintln!("Err: {}", msg); "".to_string() })
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

// Evaluation step 1: handle special forms
fn EVAL(ast: &MalType, env: &mut Env) -> Result<MalType> {
    dbg!("\n\x1b[31mEVAL\x1b[m ast:{} env:{}", ast, env);
    if ast.list().is_none() { return eval_ast(ast, env); }
    if ast.list().unwrap().is_empty() { return Ok(ast.clone()); }
    match ast.get(0).and_then(|x| x.symbol()).as_deref() {
        Some("def!") => {
            let a1 = ast.get(1).and_then(|x| x.symbol());
            let a2 = ast.get(2).clone();
            if a1.is_none() || a2.is_none() {
                return Err(malerr!("Syntax error of 'def!'."));
            }
            let (a1, a2) = (a1.unwrap(), EVAL(&a2.unwrap(), env)?);
            dbg!("EVAL def! {}->{}", a1, a2);
            Ok(env.set(&a1, &a2))
        },
        Some("let*") => {
            dbg!("EVAL let*");
            let a1 = ast.get(1)
                .and_then(|x| x.list_or_vec())
                .filter(|v| v.len() % 2 == 0);
            let a2 = ast.get(2).clone();
            if a1.is_none() || a2.is_none() {
                return Err(malerr!("Syntax error of 'let*'."));
            }
            let (a1, a2) = (a1.unwrap(), a2.unwrap());
            let mut let_env = Env::new(Some(env));
            for x in a1.chunks(2) {
                if let MalType::Symbol(s) = &x[0].clone() {
                    let v = EVAL(&x[1].clone(), &mut let_env)?;
                    let_env.set(s, &v);
                } else {
                    return Err(malerr!("List of first arg for 'let*' must be Symbols and args."));
                }
            }
            EVAL(&a2, &mut let_env)
        },
        Some("do") => {
            ast.list().unwrap().iter().skip(1).map(|x| EVAL(x, env))
                .last().unwrap_or(Ok(MalType::Nil))
        },
        Some("if") => {
            let a1 = ast.get(1);
            let a2 = ast.get(2);
            let a3 = ast.get(3);
            if a1.is_none() || a2.is_none() {
                return Err(malerr!("Syntax error of 'if'."));
            }
            let (condition, a2) = (EVAL(&a1.unwrap(), env)?, a2.unwrap());
            if !(condition == MalType::Nil || condition == MalType::False) {
                EVAL(&a2, env)
            } else {
                EVAL(&a3.unwrap_or(MalType::Nil), env)
            }
        },
        Some("fn*") => {
            dbg!("EVAL fn*1: {}", ast.clone());
            let a1 = ast.get(1)
                .and_then(|x| x.list())
                .filter(|x| x.iter()
                    .all(|x| x.symbol().is_some()));
            let a2 = ast.get(2);
            if a1.is_none() || a2.is_none() {
                return Err(malerr!("Syntax error of 'fn*'."));
            }
            let a1 = a1.unwrap().iter()
                .map(|x| x.symbol().unwrap()).collect_vec();
            let a2 = a2.unwrap();
            dbg!("EVAL fn*2: {}", MalType::Lambda(a1.clone(), vec![a2.clone()], env.clone()));
            Ok(MalType::Lambda(a1, vec![a2], env.clone()))
        },
        _ => {
            let el = eval_ast(ast, env)?;
            let f = el.get(0).unwrap();
            let args = el.list().unwrap().into_iter().skip(1).collect_vec();
            dbg!("EVAL default {}", el);
            match f {
                MalType::Fn(f) => { (f.f)(&args) },
                MalType::Lambda(bind, ast, mut lambda_env) => {
                    lambda_env.outer = Some(Rc::new(env.clone()));
                    for (b, e) in bind.iter().zip(args) {
                        lambda_env.set(&b, &e);
                    }
                    EVAL(&ast[0], &mut lambda_env)
                },
                _ => Err(malerr!("{} is not a function nor a lambda.", f)),
            }
        },
    }
}

fn PRINT(ast: &MalType) -> Result<String> {
    Ok(printer::pr_str(ast))
}

fn eval_ast(ast: &MalType, env: &mut Env) -> Result<MalType> {
    dbg!("\x1b[31m eval_ast\x1b[m ast:{}, env:{}", ast, env);
    match ast {
        MalType::Symbol(s) => {
            dbg!(" eval_ast handle Symbol {} on env:{}", s, env);
            env.get(&*s)
        },
        MalType::List(v) => {
            dbg!(" eval_ast handle List {}", ast);
            let v: Result<Vec<_>> = v.into_iter().map(|x|
                EVAL(x, env)).collect();
            Ok(MalType::List(v?))
        },
        MalType::Vec(v) => {
            let v: Result<Vec<_>> = v.into_iter().map(|x|
                EVAL(x, env)).collect();
            Ok(MalType::Vec(v?))
        },
        MalType::HashMap(v) => {
            let mut v1: Vec<(MalType, MalType)> = Vec::new();
            for (x, y) in v {
                v1.push((x.clone(), EVAL(y, env)?));
            }
            Ok(MalType::HashMap(v1))
        },
        _ => {  // other
            dbg!(" eval_ast handle default {}", ast);
            Ok(ast.clone())
        },
    }
}
