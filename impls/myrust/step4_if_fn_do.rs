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
//    dbg!("EVAL: {:?}", ast);
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
            let ast = vec![ast.unwrap()];
//            dbg!("EVAL fn* -> {}", MalType::Lambda(bind.clone(), ast.clone()));
            Ok(MalType::Lambda(bind, ast))
         },
        _ => {
//            dbg!("EVAL default");
            eval_ast(ast, repl_env)
        },
    }
}

fn PRINT(ast: &MalType) -> Result<String> {
    Ok(printer::pr_str(ast))
}

fn eval_ast(ast: &MalType, repl_env: &mut Env) -> Result<MalType> {
    match ast {
        MalType::List(v) if v.is_empty() => Ok(ast.clone()),
        MalType::List(v) => {
//            dbg!("eval_ast handle List {:?}", v);
            match v[0].clone() {
                MalType::Symbol(s) => {
                    dbg!("eval_ast List->Symbol {}", s);
                    let symbol_content = repl_env.get(&*s)?;
                    match symbol_content {
                        MalType::Fn(_) | MalType::Lambda(_, _) => {
                            let mut v1 = vec![symbol_content];
                            v1.append(&mut v.iter().skip(1).cloned().collect_vec());
                            dbg!("call EVAL {:?}", v1);
                            EVAL(&MalType::List(v1), repl_env)
                        },
                        _ => Err(malerr!("Symbol {} is not a function.", s)),
                    }
                },
                MalType::Fn(f) => {
                    dbg!("eval_ast List->Fn {}", f);
                    let v1 = v.iter().skip(1).map(|x| EVAL(x, repl_env)).collect::<Result<Vec<MalType>>>()?;
                    (f.f)(&v1).and_then(|v2| EVAL(&v2, repl_env))
                },
                MalType::Lambda(bind, ast) => {
                    let exprs = v[1..].to_vec();
                    if bind.len() != exprs.len() {
                        return Err(malerr!("Illegal number of bind:{} != exprs:{}.", bind.len(), exprs.len()));
                    }
                    dbg!("eval_ast List->Lambda bind:{:?} exprs:{:?} ast:{:?}", bind, exprs, ast);
                    let eval_exprs = exprs.iter().map(|e| EVAL(&e, repl_env)).collect::<Result<Vec<MalType>>>()?;
                    let mut temp_env = Env::new(Some(repl_env));
                    bind.iter().zip(eval_exprs).for_each(|(b, e)| temp_env.set(b, &e));
                    EVAL(&ast[0], &mut temp_env)
                },
                MalType::List(v2) => {
                    dbg!("eval_ast List->List {:?}", v2);
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
            dbg!("eval_ast handle Symbol {}", s);
            let symbol_content = repl_env.get(&*s)?;
            EVAL(&symbol_content, repl_env)
        },
        _ => {  // other
            dbg!("eval_ast handle default");
            Ok(ast.clone())
        },
    }
}
