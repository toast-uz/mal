use std::fmt;
use std::rc::Rc;
use std::hash::{Hash, Hasher};
use itertools::Itertools;
use crate::types::*;
use crate::malerr;

#[derive(Debug, Clone)]
pub struct Env {
    pub outer: Option<Rc<Env>>,
    pub data: Vec<(String, MalType)>,
}

impl Env {
    pub fn new(outer: Option<&Env>) -> Self { Self {
        outer: outer.and_then(|x| Some(Rc::from(x.clone()))),
        data: Vec::new(),
    } }

    #[allow(dead_code)]
    pub fn depth(&self) -> usize {
        let mut res = 0;
        let mut current = self.outer.clone();
        while current.is_some() {
            current = current.unwrap().outer.clone();
            res += 1;
        }
        res
    }

    // takes a symbol key and a mal value and adds to the data structure
    pub fn set(&mut self, key: &str, value: &MalType) -> MalType {
        self.remove(key);
        self.data.push((key.to_string(), value.clone()));
        MalType::Symbol(key.to_string())
    }

    pub fn remove(&mut self, key: &str) {
        if let Some(i) = self.data.iter()
                .position(|x| x.0 == key) {
            self.data.remove(self.data.len() - i);
        }
    }

    // takes a symbol key and if the current environment contains that key
    // then return the environment. If no key is found and outer is not nil
    // then call find (recurse) on the outer environment.
    fn find(&self, key: &str) -> Option<Rc<Env>> {
        let mut env = self;
        loop {
            if env.data.iter().find(|&x| x.0 == key).is_some() {
                return Some(Rc::from(env.clone()));
            }
            if env.outer.is_none() { break; }
            env = env.outer.as_ref().unwrap();
        }
        None
    }

    // takes a symbol key and uses the find method to locate the environment with the key,
    // then returns the matching value.
    // If no key is found up the outer chain, then throws/raises a "not found" error.
    pub fn get(&self, key: &str) -> Result<MalType> {
        self.find(key).and_then(|env| env.data.iter().rev()
            .find(|&x| x.0 == key)
            .map(|x| x.1.clone()))
            .ok_or_else(|| malerr!("'{}' not found.", key))
    }
}

impl fmt::Display for Env{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let env_string = format!("[{}]", self.data.iter().map(|(k, _)| k).join(" "));
        let out_string = if let Some(out) = self.outer.as_ref() {
            format!("[{}]", out.data.iter().map(|(k, _)| k).join(" "))
        } else {
            "None".to_string()
        };
        write!(f, "{{{} depth:{} out:{}}}", env_string, self.depth(), out_string)
    }
}

impl PartialEq for Env {
    fn eq(&self, other: &Self) -> bool { self.data == other.data && self.outer == other.outer }
}

impl Eq for Env { }

impl Hash for Env {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}
