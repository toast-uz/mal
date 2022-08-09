use std::ops::{Add, Sub, Mul, Div};
use std::rc::Rc;
use itertools::Itertools;
use crate::types::*;
use crate::printer;
use crate::malerr;

macro_rules! define_arithmetic_operations {
    ( $f:ident ) => ( Rc::new(move |x: &[MalType]| {
        let a = x.get(0).and_then(|x| x.num());
        let b = x.get(1).and_then(|x| x.num());
        if a.is_none() || b.is_none() {
            Err(malerr!("Illegal args for the arithmetic operation: {:?}", x))
        } else {
            let (a, b) = (a.unwrap(), b.unwrap());
            Ok(MalType::from(a.$f(b)))
        }
    }))
}

macro_rules! define_cmp_operations {
    ( $f:ident ) => ( Rc::new(move |x: &[MalType]| {
        let a = x.get(0).and_then(|x| x.num());
        let b = x.get(1).and_then(|x| x.num());
        if a.is_none() || b.is_none() {
            Err(malerr!("Illegal args for the cmp operation: {:?}", x))
        } else {
            let (a, b) = (a.unwrap(), b.unwrap());
            Ok(MalType::from(a.$f(&b)))
        }
    }) )
}

pub fn ns() -> Vec<MalFunc> {
    let mut res: Vec<MalFunc> = Vec::new();

    let add = define_arithmetic_operations!(add);
    res.push(MalFunc::new("+", add));
    let sub = define_arithmetic_operations!(sub);
    res.push(MalFunc::new("-", sub));
    let mul = define_arithmetic_operations!(mul);
    res.push(MalFunc::new("*", mul));
    let div = define_arithmetic_operations!(div);
    res.push(MalFunc::new("/", div));
    res.push(MalFunc::new("list", Rc::new(move |x: &[MalType]| {
        Ok(MalType::ListVec(MalListVec{0: true, 1: x.to_vec()}))
    })));
    res.push(MalFunc::new("list?", Rc::new(move |x: &[MalType]| {
        Ok(MalType::from(x.get(0).and_then(|x| x.list()).is_some()))
    })));
    res.push(MalFunc::new("empty?", Rc::new(move |x: &[MalType]| {
        Ok(MalType::from(x.get(0).and_then(|x| x.list_or_vec())
            .and_then(|x| Some(x.is_empty()))))
    })));
    res.push(MalFunc::new("count", Rc::new(move |x: &[MalType]| {
        Ok(MalType::from(x.get(0).and_then(|x| x.list_or_vec())
            .and_then(|x| Some(x.len()))))
    })));

    let eq = Rc::new(move |x: &[MalType]| {
        let (a, b) = (x.get(0), x.get(1));
        if a.is_none() || b.is_none() {
            Err(malerr!("Illegal args for the eq operation: {:?}", x))
        } else {
            let (a, b) = (a.unwrap(), b.unwrap());
            Ok(MalType::from(a == b))
        }
    });
    res.push(MalFunc::new("=", eq));
    let lt = define_cmp_operations!(lt);
    res.push(MalFunc::new("<", lt));
    let le = define_cmp_operations!(le);
    res.push(MalFunc::new("<=", le));
    let gt = define_cmp_operations!(gt);
    res.push(MalFunc::new(">",gt));
    let ge = define_cmp_operations!(ge);
    res.push(MalFunc::new(">=", ge));
    res.push(MalFunc::new("pr-str", Rc::new(move |x: &[MalType]| {
        Ok(MalType::String(x.iter().map(|x| printer::pr_str(x, true)).join(" ")))
    })));
    res.push(MalFunc::new("str", Rc::new(move |x: &[MalType]| {
        Ok(MalType::String(x.iter().map(|x| printer::pr_str(x, false)).join("")))
    })));
    res.push(MalFunc::new("prn", Rc::new(move |x: &[MalType]| {
        let s = MalType::Print(x.iter().map(|x| printer::pr_str(x, true)).join(" "));
        println!("{}", s);
        Ok(MalType::Nil)
    })));
    res.push(MalFunc::new("println", Rc::new(move |x: &[MalType]| {
        let s = MalType::Print(x.iter().map(|x| printer::pr_str(x, false)).join(" "));
        println!("{}", s);
        Ok(MalType::Nil)
    })));
    res
}