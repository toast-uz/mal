use std::error;
use std::fmt;
use std::rc::Rc;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::ops::{Add, Sub, Mul, Div};
use itertools::Itertools;
use regex::Regex;
use once_cell::sync::Lazy;
use crate::env::*;

const PREFIX_KEYWORD: &str = "\u{029e}";

const _NAME2MALTYPE: [(&str, &MalType); 9] = [
    ("(", &MalType::Lparen),
    (")", &MalType::Rparen),
    ("[", &MalType::Lsqure),
    ("]", &MalType::Rsqure),
    ("{", &MalType::Lcurly),
    ("}", &MalType::Rcurly),
    ("nil", &MalType::Nil),
    ("true", &MalType::True),
    ("false", &MalType::False),
];

static MALTYPE2NAME: Lazy<HashMap<&'static MalType, &'static str>> = Lazy::new(||
    _NAME2MALTYPE.iter().cloned().map(|(k, v)| (v, k)).collect());

static NAME2MALTYPE: Lazy<HashMap<&'static str, &'static MalType>> = Lazy::new(||
    _NAME2MALTYPE.iter().cloned().collect());

static STRING_RE: Lazy<Regex> = Lazy::new(||
    Regex::new(r#""(?:\\.|[^\\"])*"?"#).unwrap());

pub type Result<T> = std::result::Result<T, MalError>;

#[macro_export]
macro_rules! malerr {
    ( $( $x:expr ),* ) => (
        MalError::new(&format!($( $x ),* ))
    )
}

// ----------- Token -----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    pub fn new(s: &str) -> Self { Self(s.to_string()) }
    pub fn to_string(&self) -> String { self.0.clone() }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// ----------- Number ------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Int(i64), Float(f64),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Int(num) => format!("{}", num),
            Self::Float(num) => format!("{}", num),
        };
        write!(f, "{}", s)
    }
}

impl Eq for Number { }

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (*self, *other) {
            (Number::Int(x), Number::Int(y)) => x.partial_cmp(&y),
            (Number::Int(x), Number::Float(y)) => (x as f64).partial_cmp(&y),
            (Number::Float(x), Number::Int(y)) => x.partial_cmp(&(y as f64)),
            (Number::Float(x), Number::Float(y)) => x.partial_cmp(&y),
        }
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()    // ignore nan
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Int(num) => num.hash(state),
            Self::Float(num) => (*num as i64).hash(state),
        }
    }
}

impl From<i64> for Number { fn from(x: i64) -> Self { Self::Int(x) } }
impl From<f64> for Number { fn from(x: f64) -> Self { Self::Float(x) } }
impl From<usize> for Number { fn from(x: usize) -> Self { Self::from(x as i64) } }

impl std::str::FromStr for Number {
    type Err = MalError;

    fn from_str(s: &str) -> Result<Self> {
        if let Ok(x) = s.parse::<i64>() {
            return Ok(Number::Int(x))
        }
        if let Ok(x) = s.parse::<f64>() {
            return Ok(Number::Float(x))
        }
        Err(malerr!("Cannot parse to Number from string."))
    }
}

macro_rules! define_arithmetic_operations {
    ( $t:ident, $f:ident ) => ( impl $t for Number { type Output = Self;
        fn $f(self, other: Self) -> Self::Output {
            match (self, other) {
                (Number::Int(x), Number::Int(y)) => Number::Int(x.$f(&y)),
                (Number::Int(x), Number::Float(y)) => Number::Float((x as f64).$f(&y)),
                (Number::Float(x), Number::Int(y)) => Number::Float(x.$f(&(y as f64))),
                (Number::Float(x), Number::Float(y)) => Number::Float(x.$f(&y)),
            }
        }
    })
}

define_arithmetic_operations!(Add, add);
define_arithmetic_operations!(Sub, sub);
define_arithmetic_operations!(Mul, mul);
define_arithmetic_operations!(Div, div);

// ----------- MalType -----------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MalType {
    Comment, Nil, True, False, Num(Number),
    Lparen, Rparen, Lsqure, Rsqure, Lcurly, Rcurly,
    String(String), Keyword(String), Symbol(String), Print(String),
    ListVec(MalListVec), HashMap(Vec<(MalType, MalType)>),
    Fn(MalFunc), Lambda(Vec<String>, Vec<MalType>, Env),
}

impl MalType {
    pub fn new_vec(typ: &str, v: &[MalType]) -> Self {
        match typ {
            "list" => MalType::ListVec(MalListVec{0: true, 1: v.to_vec()}),
            "vec" => MalType::ListVec(MalListVec{0: false, 1: v.to_vec()}),
            "hashmap" => {
                let mut v1: Vec<(MalType, MalType)> = Vec::new();
                for x in v.chunks(2) {
                    v1.push((x[0].clone(), x[1].clone()));
                }
                MalType::HashMap(v1)
            },
            _ => unreachable!(),
        }
    }

    pub fn list(&self) -> Option<Vec<MalType>> {
        match self {
            MalType::ListVec(mlv) if mlv.0 => Some(mlv.1.clone()),
            _ => None,
        }
    }

    pub fn list_or_vec(&self) -> Option<Vec<MalType>> {
        match self {
            MalType::ListVec(mlv) => Some(mlv.1.clone()),
            _ => None,
        }
    }

    pub fn symbol(&self) -> Option<String> {
        match self {
            MalType::Symbol(s) => Some(s.to_string()),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<String> {
        match self {
            MalType::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    pub fn num(&self) -> Option<Number> {
        match *self {
            MalType::Num(num) => Some(num),
            _ => None,
        }
    }

    pub fn get(&self, i: usize) -> Option<MalType> {
        self.list_or_vec().and_then(|v| v.get(i).cloned())
    }

    pub fn from_token(token: &Token) -> Result<Self> {
        let s = token.to_string();
        if let Some(';') = s.chars().next() {
            Ok(Self::Comment)
        } else if let Ok(num) = s.parse::<Number>() {
            Ok(Self::Num(num))
        } else if STRING_RE.is_match(&s) {
            Ok(Self::String(Self::_unescape(&s[1..(s.len() - 1)])))
        } else if let Some('\"') = s.chars().next() {
            Err(malerr!("expected '\"', got EOF"))
        } else if let Some(':') = s.chars().next() {
            Ok(Self::Keyword(s[1..].to_string()))
        } else { match &s as &str {
            x if NAME2MALTYPE.contains_key(x) => Ok(NAME2MALTYPE[x].clone()),
            x => Ok(Self::Symbol(x.to_string())),
        } }
    }

    fn _unescape(s: &str) -> String {
        s.replace(r"\\", PREFIX_KEYWORD)
            .replace(r"\n", "\n")
            .replace(r#"\""#, "\"")
            .replace(PREFIX_KEYWORD, r"\")
            .to_string()
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Comment => "".to_string(),
            x if MALTYPE2NAME.contains_key(&x) => MALTYPE2NAME[&*x].to_string(),
            Self::Num(num) => format!("{}", num),
            Self::String(s) => s.to_string(),
            Self::Keyword(s) => format!(":{}", s),
            Self::Symbol(s) => s.to_string(),
            Self::Print(s) => s.to_string(),
            Self::ListVec(v) => format!("{}", v),
            Self::HashMap(v) =>
                format!("{{{}}}", v.iter().map(|(k, v)| vec![k, v]).flatten().join(" ")),
            Self::Fn(f) => format!("#<{}>", f.name),
            Self::Lambda(args, v, env) =>
                format!("#<lambda:({})->{{{}}}; env_depth:{}>", args.iter().join(" "), v.get(0).unwrap(), env.depth()),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<bool> for MalType { fn from(x: bool) -> Self {
    if x { Self::True } else { Self::False }
} }
impl From<Option<bool>> for MalType { fn from(x: Option<bool>) -> Self {
    if let Some(x) = x { Self::from(x) } else { Self::False }
} }
impl From<Option<usize>> for MalType { fn from(x: Option<usize>) -> Self {
    if let Some(x) = x { Self::from(x) } else { Self::from(0usize) }
} }
impl From<Number> for MalType { fn from(x: Number) -> Self { Self::Num(x) } }
impl From<i64> for MalType { fn from(x: i64) -> Self { Self::from(Number::from(x)) } }
impl From<f64> for MalType { fn from(x: f64) -> Self { Self::from(Number::from(x)) } }
impl From<usize> for MalType { fn from(x: usize) -> Self { Self::from(Number::from(x)) } }

unsafe impl Send for MalType {}
unsafe impl Sync for MalType {}

// ----------- MalListVec -----------

#[derive(Debug, Clone, Eq, Hash)]
pub struct MalListVec(pub bool, pub Vec<MalType>);  // true = List, false = Vec

impl PartialEq for MalListVec {
    fn eq(&self, other: &Self) -> bool { self.1 == other.1 }
}

impl fmt::Display for MalListVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 {
            write!(f, "({})", self.1.iter().join(" "))
        } else {
            write!(f, "[{}]", self.1.iter().join(" "))
        }
    }
}

// ----------- MalFunc -----------

pub type Func<T> = Rc<dyn Fn(&[T]) -> Result<T>>;

#[derive(Clone)]
pub struct MalFunc{
    pub name: String,
    pub f: Func<MalType>,
}

#[allow(dead_code)]
impl MalFunc{
    pub fn new(name: &str, f: Func<MalType>) -> Self {
        Self{ name: name.to_string(), f: f.clone() }
    }
}

impl fmt::Display for MalFunc{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for MalFunc{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for MalFunc {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Eq for MalFunc { }

impl Hash for MalFunc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

unsafe impl Send for MalFunc {}
unsafe impl Sync for MalFunc {}

// ----------- MalError -----------

#[derive(Debug, Clone)]
pub struct MalError {
    msg: String,
}

impl MalError {
    pub fn new(msg: &str) -> Self{ Self{msg: msg.to_string()} }
}

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

// This is important for other errors to wrap this one.
impl error::Error for MalError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
