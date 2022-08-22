use std::collections::HashMap;
use itertools::Itertools;
use regex::Regex;
use once_cell::sync::Lazy;
use crate::types::*;
use crate::malerr;

static NAME2SYMBOL: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("'", "quote"),
        ("`", "quasiquote"),
        ("~", "unquote"),
        ("~@", "splice-unquote"),
        ("^", "with-meta"),
        ("@", "deref"),
    ].iter().cloned().collect()});

static RE: Lazy<Regex> = Lazy::new(||
    Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap());

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
    RE.captures_iter(s.trim())
        .map(|cap| Token::new(&cap[1])).collect_vec()
}

// look at the contents of the token
// and return the appropriate scalar (simple/single) data type value.
fn read_atm(reader: &mut Reader) -> Result<MalType> {
    MalType::from_token(&reader.next().unwrap())
}

fn read_sequence(reader: &mut Reader, typ: &str, start: &MalType, end: &MalType)
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
    read_sequence(reader, "list", &MalType::Lparen, &MalType::Rparen)
}

fn read_vector(reader: &mut Reader) -> Result<MalType> {
    read_sequence(reader, "vec", &MalType::Lsqure, &MalType::Rsqure)
}

fn read_hash_map(reader: &mut Reader) -> Result<MalType> {
    read_sequence(reader, "hashmap", &MalType::Lcurly, &MalType::Rcurly)
}

// peek at the first token in the Reader object
// and switch on the first character of that token.
// The return value from read_form is a mal data type.
pub fn read_form(reader: &mut Reader) -> Result<MalType> {
    match MalType::from_token(&reader.peek().unwrap())? {
        MalType::Symbol(x) if NAME2SYMBOL.contains_key(&*x) => {
            reader.next();
            if NAME2SYMBOL[&*x] == "with-meta" {
                let meta = read_form(reader)?;
                Ok(MalType::ListVec(MalListVec{0: true,
                    1: vec![MalType::Symbol(NAME2SYMBOL[&*x].to_string()),
                    read_form(reader)?, meta]}))
            } else {
                Ok(MalType::ListVec(MalListVec{0: true,
                    1: vec![MalType::Symbol(NAME2SYMBOL[&*x].to_string()),
                    read_form(reader)?]}))
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
//    eprintln!("{} {:?}", s, tokens);
    // if tokens.is_empty() { return Err("Blank Line") }
    read_form(&mut Reader::new(&tokens))
}
