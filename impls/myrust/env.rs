use std::rc::Rc;
use crate::types::*;
use crate::malerr;

#[derive(Debug, Clone)]
pub struct Env {
    outer: Option<Rc<Env>>,
    data: Vec<(String, MalType)>,
}

impl Env {
    pub fn new(outer: Option<&Env>) -> Self { Self {
        outer: outer.and_then(|x| Some(Rc::from(x.clone()))),
        data: Vec::new(),
    } }

    // takes a symbol key and a mal value and adds to the data structure
    pub fn set(&mut self, key: &str, value: &MalType) {
        self.data.push((key.to_string(), value.clone()));
    }

    pub fn remove(&mut self, key: &str) {
        let i = self.data.len() - self.data.iter()
            .position(|x| x.0 == key)
            .expect("Cannot remove by undefined key.");
        self.data.remove(i);
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