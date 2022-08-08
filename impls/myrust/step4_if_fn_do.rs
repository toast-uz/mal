#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;
mod env;
mod core;

use std::io::{stdin, stdout, Write};
use itertools::Itertools;
use types::*;
use env::*;

const DEBUG: bool = true;
macro_rules! dbg {( $( $x:expr ),* ) => ( if DEBUG {eprintln!($( $x ),* )})}

 fn main() {
    let mut repl_env: Env = Env::new(None);
    core::ns().iter().for_each(|f|
        repl_env.set(&f.name, &MalType::Fn(f.clone())));
    loop {
        let mut s = String::new();
        print!("user> "); stdout().flush().unwrap();
        if let Ok(0) = stdin().read_line(&mut s) { break; }
        if !s.trim().is_empty() {
            println!("{}", rep(&s, &mut repl_env));
        }
    }
}

fn rep(s: &str, repl_env: &mut Env) -> String {
    READ(s).and_then(|x| EVAL(&x, repl_env)).and_then(|x| PRINT(&x))
        .unwrap_or_else(|msg| { eprintln!("Err: {}", msg); "".to_string() })
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

// Evaluation step 1: handle special forms
fn EVAL(ast: &MalType, repl_env: &mut Env) -> Result<MalType> {
    dbg!("\n\x1b[31mEVAL\x1b[m ast:{} env:{}", ast, repl_env.depth());
    match ast.get(0).and_then(|x| x.symbol()).as_deref() {
        Some("def!") => {
            let key = ast.get(1).and_then(|x| x.symbol());
            let value = ast.get(2).clone();
            if key.is_none() || value.is_none() {
                return Err(malerr!("Syntax error of 'def!'."));
            }
            let (key, value) = (key.unwrap(), EVAL(&value.unwrap(), repl_env)?);
            if let MalType::Fn(f) = value.clone() {
                repl_env.remove(&f.name);
                repl_env.set(&key, &MalType::Fn(MalFunc::new(&key, f.f)));
            } else {
                repl_env.set(&key, &value);
            }
            dbg!("EVAL def! {}->{}", key, value);
            EVAL(&MalType::Symbol(key), repl_env)
        },
        Some("let*") => {
            dbg!("EVAL let*");
            let keys = ast.get(1)
                .and_then(|x| x.list_or_vec())
                .filter(|v| v.len() % 2 == 0);
            let value = ast.get(2).clone();
            if keys.is_none() || value.is_none() {
                return Err(malerr!("Syntax error of 'let*'."));
            }
            let (keys, value) = (keys.unwrap(), value.unwrap());
            let mut new_env = Env::new(Some(repl_env));
            for x in keys.chunks(2) {
                if let MalType::Symbol(s) = &x[0].clone() {
                    let v = EVAL(&x[1].clone(), &mut new_env)?;
                    new_env.set(s, &v);
                } else {
                    return Err(malerr!("List of first arg for 'let*' or 'fn*' must be Symbols and args."));
                }
            }
            EVAL(&value, &mut new_env)
        },
        Some("do") => {
            let v = ast.list().unwrap();
            let mut res = MalType::Nil;
            for x in &v[1..] { res = EVAL(x, repl_env)?; }
            Ok(res)
        },
        Some("if") => {
            let condition = ast.get(1);
            let true_action = ast.get(2);
            let false_action = ast.get(3).unwrap_or(MalType::Nil);
            if condition.is_none() || true_action.is_none() {
                return Err(malerr!("Syntax error of 'if'."));
            }
            let (condition, true_action) = (EVAL(&condition.unwrap(), repl_env)?, true_action.unwrap());
            if !(condition == MalType::Nil || condition == MalType::False) {
                EVAL(&true_action, repl_env)
            } else {
                EVAL(&false_action, repl_env)
            }
        },
        Some("fn*") => {
            dbg!("EVAL fn*1: {}", ast.clone());
            let bind = ast.get(1)
                .and_then(|x| x.list())
                .filter(|x| x.iter()
                    .all(|x| x.symbol().is_some()));
            let ast = ast.get(2);
            if bind.is_none() || ast.is_none() {
                return Err(malerr!("Syntax error of 'fn*'."));
            }
            let bind = bind.unwrap().iter()
                .map(|x| x.symbol().unwrap()).collect_vec();
            dbg!("EVAL fn*2: {}", MalType::Lambda(bind.clone(), vec![ast.clone().unwrap()]));
            EVAL(&MalType::Lambda(bind, vec![ast.unwrap()]), repl_env)
        },
        _ => {
            dbg!("EVAL default");
            eval_ast(ast, repl_env)
        },
    }
}

fn PRINT(ast: &MalType) -> Result<String> {
    Ok(printer::pr_str(ast))
}

fn eval_ast(ast: &MalType, repl_env: &mut Env) -> Result<MalType> {
    dbg!("\x1b[31m eval_ast\x1b[m ast:{}, env:{}", ast, repl_env.depth());
    match ast {
        MalType::List(v) if v.is_empty() => Ok(ast.clone()),
        MalType::List(v) => {
            match v[0].clone() {
                MalType::Symbol(s) => {
                    dbg!(" eval_ast List->Symbol {}", s);
                    let symbol_content = repl_env.get(&*s)?;
                    match symbol_content {
                        MalType::Fn(_) | MalType::Lambda(_, _) => {
                            let mut v1 = vec![symbol_content];
                            v1.append(&mut v.iter().skip(1).cloned().collect_vec());
                            dbg!(" call EVAL {:?}", v1);
                            EVAL(&MalType::List(v1), repl_env)
                        },
                        _ => Err(malerr!("Symbol {} is not a function.", s)),
                    }
                },
                MalType::Fn(f) => {
                    dbg!(" eval_ast List->Fn {}", f);
                    let v1 = v.iter().skip(1).map(|x| EVAL(x, repl_env)).collect::<Result<Vec<MalType>>>()?;
                    (f.f)(&v1).and_then(|v2| EVAL(&v2, repl_env))
                },
                MalType::Lambda(bind, ast) => {
                    // Make exprs by args.
                    let exprs = v[1..].to_vec();
                    if bind.len() != exprs.len() {
                        return Err(malerr!("Illegal number of bind:{} != exprs:{}.", bind.len(), exprs.len()));
                    }
                    dbg!("\x1b[32m eval_ast List->Lambda#1\x1b[m bind:{:?} = exprs:{:?} ast[0]:{}", bind, exprs, ast[0]);
                    // Each bind links with each evaluated expr.
                    let exprs = exprs.iter().map(|e| EVAL(&e, repl_env)).collect::<Result<Vec<MalType>>>()?;
                    // Binds make new environment.
                    let mut new_env = Env::new(Some(repl_env));
                    bind.iter().zip(exprs).for_each(|(b, e)| new_env.set(b, &e));
                    dbg!("\x1b[32m eval_ast List->Lambda#2\x1b[m bind:{:?} ast[0]:{} env:{}", bind, ast[0], new_env.depth());
                    // Evaluate ast[0] under new environment.
                    let tmp = EVAL(&ast[0], &mut new_env);
                    dbg!("\x1b[32m eval_ast List->Lambda#3\x1b[m res:{:?} env:{}", tmp, new_env.depth());
                    tmp
                },
                MalType::List(v2) => {
                    dbg!(" eval_ast List->List {:?}", v2);
                    let mut v1 = vec![EVAL(&MalType::List(v2), repl_env)?];
                    v1.append(&mut v.iter().skip(1).cloned().collect_vec());
                    EVAL(&MalType::List(v1), repl_env)
                },
                _ => Ok(ast.clone()),
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
            dbg!(" eval_ast handle Symbol {}", s);
            let symbol_content = repl_env.get(&*s)?;
            EVAL(&symbol_content, repl_env)
        },
        _ => {  // other
            dbg!(" eval_ast handle default {}", ast);
            Ok(ast.clone())
        },
    }
}
