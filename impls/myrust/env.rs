use std::collections::HashMap;
use crate::types::*;
use crate::malerr;

#[derive(Debug, Clone)]
pub struct Env<'a> {
    outer: Option<&'a Env<'a>>,
    data: HashMap<&'a str, &'a MalFunc<'a>>,
}

impl<'a> Env<'a> {
    pub fn new(outer: Option<&'a Env>) -> Self { Self { outer: outer, data: HashMap::new(), } }

    // takes a symbol key and a mal value and adds to the data structure
    pub fn set(&mut self, key: &'a str, value: &'a MalFunc<'a>) {
        self.data.insert(key, value);
    }

    // takes a symbol key and if the current environment contains that key
    // then return the environment. If no key is found and outer is not nil
    // then call find (recurse) on the outer environment.
    fn find(&self, key: &str) -> Option<&'a Env> {
        let env = Env::new(Some(self));
        while let Some(env) = env.outer {
            if env.data.contains_key(key) { return Some(env); }
        }
        None
    }

    // takes a symbol key and uses the find method to locate the environment with the key,
    // then returns the matching value.
    // If no key is found up the outer chain, then throws/raises a "not found" error.
    pub fn get(&self, key: &str) -> Result<&MalFunc> {
        self.find(key).and_then(|env| env.data.get(key).cloned())
            .ok_or_else(|| malerr!("{} is not found.", key))
    }
}