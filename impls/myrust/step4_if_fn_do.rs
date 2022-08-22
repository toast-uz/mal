#![allow(non_snake_case)]
mod reader;
mod printer;
mod types;
mod env;
mod core;

use std::rc::Rc;
use itertools::Itertools;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use types::*;
use env::*;

#[macro_use]
extern crate lazy_static;

const DEBUG: bool = false;
macro_rules! dbg {( $( $x:expr ),* ) => ( if DEBUG {eprintln!($( $x ),* )})}

 fn main() {
    let mut rl = Editor::<()>::new().unwrap();
    if rl.load_history(".mal-history").is_err() {
        eprintln!("No previous history.");
    }

    let mut env: Env = Env::new(None);
    core::ns().iter().for_each(|f|
        { env.set(&f.name, &MalType::Fn(f.clone())); });
    rep("(def! not (fn* (a) (if a false true)))", &mut env).unwrap();

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                rl.save_history(".mal-history").unwrap();
                if line.len() > 0 {
                    match rep(&line, &mut env) {
                        Ok(out) => println!("{}", out),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            },
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn rep(s: &str, env: &mut Env) -> Result<String> {
    READ(s).and_then(|x| EVAL(&x, env)).and_then(|x| PRINT(&x))
}

fn READ(s: &str) -> Result<MalType> {
    reader::read_str(s)
}

// Evaluation step 1: handle special forms
fn EVAL(ast: &MalType, env: &mut Env) -> Result<MalType> {
    dbg!("\n\x1b[31mEVAL\x1b[m ast:{} env:{}", ast, env);
    if ast.list().is_none() { return eval_ast(ast, env); }
    if ast.list().unwrap().is_empty() { return Ok(ast.clone()); }
    dbg!("EVAL match...");
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
                .and_then(|x| x.list_or_vec())
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
            let args = el.list_or_vec().unwrap().into_iter().skip(1).collect_vec();
            dbg!("EVAL default {}", el);
            match f {
                MalType::Fn(f) => { (f.f)(&args) },
                MalType::Lambda(bind, ast, mut lambda_env) => {
                    lambda_env.outer = Some(Rc::new(env.clone()));
                    for i in 0..bind.len() {
                        if bind[i] == "&" { if i < bind.len() - 1 {
                            lambda_env.set(&bind[i + 1],
                                &MalType::ListVec(MalListVec{0: true,
                                    1: args.iter().skip(i).cloned().collect_vec()}));
                            } break;
                        }
                        if i >= args.len() { break; }
                        lambda_env.set(&bind[i], &args[i]);
                    }
                    EVAL(&ast[0], &mut lambda_env)
                },
                _ => Err(malerr!("{} is not a function nor a lambda.", f)),
            }
        },
    }
}

fn PRINT(ast: &MalType) -> Result<String> {
    Ok(printer::pr_str(ast, true))
}

fn eval_ast(ast: &MalType, env: &mut Env) -> Result<MalType> {
    dbg!("\x1b[31m eval_ast\x1b[m ast:{}, env:{}", ast, env);
    match ast {
        MalType::Symbol(s) => {
            dbg!(" eval_ast handle Symbol {} on env:{}", s, env);
            env.get(&*s)
        },
        MalType::ListVec(v) => {
            dbg!(" eval_ast handle List or Vec {}", ast);
            let v1: Vec<MalType> = v.1.clone().into_iter().map(|x|
                EVAL(&x, env)).collect::<Result<Vec<MalType>>>()?;
            Ok(MalType::ListVec(MalListVec{0: v.0, 1: v1}))
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
