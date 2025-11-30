use crate::parser::Command;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Functions {
    funcs: HashMap<String, Vec<Vec<Command>>>,
}

impl Functions {
    pub fn new() -> Self {
        Self {
            funcs: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: String, body: Vec<Vec<Command>>) {
        self.funcs.insert(name, body);
    }

    pub fn get(&self, name: &str) -> Option<&Vec<Vec<Command>>> {
        self.funcs.get(name)
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, name: &str) {
        self.funcs.remove(name);
    }
}
