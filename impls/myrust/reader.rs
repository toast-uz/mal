use std::collections::HashMap;
use itertools::Itertools;
use regex::Regex;
use crate::types::*;
use crate::malerr;

#[derive(Debug, Clone)]
pub struct Reader {
    tokens: Vec<Token>,
    pos: usize,
}

impl Reader {
    fn new(s: &[Token]) -> Self {
        Self { tokens: s.to_vec(), pos: 0, }
    }

    // just returns the token at the current position.
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    // returns the token at the current position and increments the position
    fn next(&mut self) -> Option<Token> {
        self.pos += 1;
        self.tokens.get(self.pos - 1).cloned()
    }
}

// take a single string and return an array/list of all the tokens (strings) in it.
fn tokenize(s: &str) -> Vec<Token> {
    let re = Regex::new(r"[\s,]*(~@|[\[\]{}()'`~^@]|\x22(?:[\\].|[^\\\x22])*\x22?|;.*|[^\s\[\]{}()'\x22`@,;]+)").unwrap();
    re.captures_iter(s)
        .map(|cap| Token::new(&cap[1]))
        .collect_vec()
}

// look at the contents of the token
// and return the appropriate scalar (simple/single) data type value.
fn read_atm(reader: &mut Reader) -> Result<MalType> {
    MalType::from_token(&reader.next().unwrap())
}

fn read_sequence(reader: &mut Reader, typ: &MalType, start: &MalType, end: &MalType)
        -> Result<MalType> {
    let token = reader.next();
    if token.is_none() || MalType::from_token(&token.unwrap())? != *start {
        return Err(malerr!("expected '{}'", *start));
    }
    let mut res: Vec<MalType> = vec![];
    let mut token = reader.peek();
    loop {
        if token.is_none() { return Err(malerr!("expected '{}', got EOF", *end)) }
        if MalType::from_token(&token.unwrap())? == *end { break; }
        res.push(read_form(reader)?);
        token = reader.peek();
    }
    reader.next();
    Ok(MalType::new_vec(typ, &res))
}

// repeatedly call read_form with the Reader object
// until it encounters a ')' token
// (if it reach EOF before reading a ')' then that is an error).
// It accumulates the results into a List type.
fn read_list(reader: &mut Reader) -> Result<MalType> {
    read_sequence(reader, &MalType::List(Vec::new()),
        &MalType::Lparen, &MalType::Rparen)
}

fn read_vector(reader: &mut Reader) -> Result<MalType> {
    read_sequence(reader, &MalType::Vec(Vec::new()),
        &MalType::Lsqure, &MalType::Rsqure)
}

fn read_hash_map(reader: &mut Reader) -> Result<MalType> {
    read_sequence(reader, &MalType::HashMap(Vec::new()),
        &MalType::Lcurly, &MalType::Rcurly)
}

const NAME2SYMBOL: [(&str, &str); 6] = [
    ("'", "quote"),
    ("`", "quasiquote"),
    ("~", "unquote"),
    ("~@", "splice-unquote"),
    ("^", "with-meta"),
    ("@", "deref"),
];

// peek at the first token in the Reader object
// and switch on the first character of that token.
// The return value from read_form is a mal data type.
pub fn read_form(reader: &mut Reader) -> Result<MalType> {
    let name2symbol: HashMap<&str, &str> = NAME2SYMBOL.iter().cloned().collect();
    match MalType::from_token(&reader.peek().unwrap())? {
        MalType::Symbol(x) if name2symbol.contains_key(&*x) => {
            reader.next();
            if name2symbol[&*x] == "with-meta" {
                let meta = read_form(reader)?;
                Ok(MalType::List(vec![MalType::Symbol(name2symbol[&*x].to_string()),
                    read_form(reader)?, meta]))
            } else {
                Ok(MalType::List(vec![MalType::Symbol(name2symbol[&*x].to_string()),
                    read_form(reader)?]))
            }
        },
        MalType::Comment => { reader.next(); Ok(MalType::Comment) },
        MalType::Rparen => Err(malerr!("unexpected ')'")),
        MalType::Lparen => { read_list(reader) },
        MalType::Rsqure => Err(malerr!("unexpected ']'")),
        MalType::Lsqure => { read_vector(reader) },
        MalType::Rcurly => Err(malerr!("unexpected '}}'")),
        MalType::Lcurly => { read_hash_map(reader) },
        _ => read_atm(reader),
    }
}

// call tokenize and then create a new Reader object instance with the tokens.
// Then it will call read_form with the Reader instance.
pub fn read_str(s: &str) -> Result<MalType> {
    let tokens = tokenize(s);
    // if tokens.is_empty() { return Err("Blank Line") }
    read_form(&mut Reader::new(&tokens))
}
