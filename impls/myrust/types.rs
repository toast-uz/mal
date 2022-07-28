use std::error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use itertools::Itertools;
use regex::Regex;

const PREFIX_KEYWORD: &str = "\u{029e}";

const NAME2MALTYPE: [(&'static str, &MalType); 9] = [
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

type Result<T> = std::result::Result<T, MalError>;

#[macro_export]
macro_rules! malerr {
    ( $( $x:expr ),* ) => (
        MalError::new(&format!($( $x ),* ))
    )
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum MalType {
    Comment, Nil, True, False, Int(i64), Float(f64),
    Lparen, Rparen, Lsqure, Rsqure, Lcurly, Rcurly,
    String(String), Keyword(String), Symbol(String),
    List(Vec<MalType>), Vec(Vec<MalType>), HashMap(HashMap<MalType, MalType>),
}

impl MalType {
    pub fn new_vec(typ: &MalType, v: &[MalType]) -> Self {
        match typ {
            MalType::List(_) => MalType::List(v.to_vec()),
            MalType::Vec(_) => MalType::Vec(v.to_vec()),
            MalType::HashMap(_) => {
                let mut hm: HashMap<MalType, MalType> = HashMap::new();
                for x in v[..].chunks(2) {
                    hm.insert(x[0].clone(), x[1].clone());
                }
                MalType::HashMap(hm)
            },
            _ => unreachable!(),
        }
    }

    pub fn from_token(token: &Token) -> Result<Self> {
        let s = token.to_string();
        let name2maltype: HashMap<&str, &MalType> = NAME2MALTYPE.iter().cloned().collect();
        let string_re = Regex::new(r"\x22(?:[\\].|[^\\\x22])*\x22").unwrap();
        if let Some(';') = s.chars().next() {
            Ok(Self::Comment)
        } else if let Ok(num) = s.parse::<i64>() {
            Ok(Self::Int(num))
        } else if let Ok(num) = s.parse::<f64>() {
            Ok(Self::Float(num))
        } else if string_re.is_match(&s) {
            Ok(Self::String(Self::_unescape(&s[1..(s.len() - 1)])))
        } else if let Some('\"') = s.chars().next() {
            Err(malerr!("expected '\"', got EOF"))
        } else if let Some(':') = s.chars().next() {
            Ok(Self::Keyword(s[1..].to_string()))
        } else { match &s as &str {
            x if name2maltype.contains_key(x) => Ok(name2maltype[x].clone()),
            x => Ok(Self::Symbol(x.to_string())),
        } }
    }

    fn _unescape(s: &str) -> String {
        let re1 = Regex::new(r"\\x22").unwrap();  // replace to temp
        let re2 = Regex::new(r"\n").unwrap();
        let re3 = Regex::new(r"\\").unwrap();
        let re4 = Regex::new(PREFIX_KEYWORD).unwrap(); // replace from temp

        // a backslash followed by a doublequote is
        // translated into a plain doublequote character,
        let res1 = re1.replace(s, PREFIX_KEYWORD);
        // a backslash followed by "n" is translated into a newline,
        let res2 = re2.replace(&res1, "\n");
        // a backslash followed by another backslash is
        // translated into a single backslash.
        let res3 = re3.replace(&res2, r"\");
        let res4 = re4.replace(&res3, "\"");
        res4.to_string()
    }

    pub fn to_string(&self) -> String {
        let maltype2name: HashMap<&MalType, &str> = NAME2MALTYPE.iter().cloned()
            .map(|(k, v)| (v, k)).collect();
        match self {
            Self::Comment => "".to_string(),
            x if maltype2name.contains_key(&x) => maltype2name[&*x].to_string(),
            Self::Int(num) => format!("{}", num),
            Self::Float(num) => format!("{}", num),
            Self::String(s) => format!("\"{}\"", s),
            Self::Keyword(s) => format!(":{}", s),
            Self::Symbol(s) => s.to_string(),
            Self::List(v) =>
                format!("({})", v.iter().map(|x| x.to_string()).join(" ")),
            Self::Vec(v) =>
                format!("[{}]", v.iter().map(|x| x.to_string()).join(" ")),
            Self::HashMap(hm) =>
                format!("{{{}}}",
                    hm.iter().map(|(k, v)| vec![k, v]).flatten().join(" ")),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Eq for MalType { }

impl Hash for MalType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Int(num) => num.hash(state),
            Self::Float(num) => (*num as i64).hash(state),
            Self::String(s) => s.hash(state),
            Self::Keyword(s) => s.hash(state),
            Self::Symbol(s) => s.hash(state),
            Self::List(v) => v.hash(state),
            Self::Vec(v) => v.hash(state),
            Self::HashMap(hm) => {
                let v = hm.iter().map(|(k, v)| vec![k, v])
                    .flatten().collect_vec();
                v.hash(state)
            },
            x => std::mem::discriminant(x).hash(state),
        };
    }
}

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