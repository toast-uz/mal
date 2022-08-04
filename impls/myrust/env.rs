use std::collections::HashMap;
use crate::types::*;
use crate::malerr;

#[derive(Debug, Clone)]
pub struct Env<'a> {
    outer: Option<&'a Env<'a>>,
    data: HashMap<String, MalType>,
}

impl<'a> Env<'a> {
    pub fn new(outer: Option<&'a Env>) -> Self { Self { outer: outer, data: HashMap::new(), } }

    // takes a symbol key and a mal value and adds to the data structure
    pub fn set(&mut self, key: &str, value: &MalType) {
        self.data.insert(key.to_string(), value.clone());
    }

    // takes a symbol key and if the current environment contains that key
    // then return the environment. If no key is found and outer is not nil
    // then call find (recurse) on the outer environment.
    fn find(&self, key: &str) -> Option<&'a Env> {
        let mut env = self;
        loop {
            if env.data.contains_key(key) { return Some(env); }
            if env.outer.is_none() { break; }
            env = env.outer.unwrap();
        }
        None
    }

    // takes a symbol key and uses the find method to locate the environment with the key,
    // then returns the matching value.
    // If no key is found up the outer chain, then throws/raises a "not found" error.
    pub fn get(&self, key: &str) -> Result<MalType> {
        self.find(key).and_then(|env| env.data.get(key).cloned())
            .ok_or_else(|| malerr!("'{}' not found.", key))
    }
}