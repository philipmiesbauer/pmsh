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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Command, SimpleCommand};

    fn create_dummy_body(name: &str) -> Vec<Vec<Command>> {
        vec![vec![Command::Simple(SimpleCommand {
            name: name.to_string(),
            args: vec![],
            assignments: vec![],
        })]]
    }

    #[test]
    fn test_new() {
        let functions = Functions::new();
        assert!(functions.funcs.is_empty());
    }

    #[test]
    fn test_set_and_get() {
        let mut functions = Functions::new();
        let body = create_dummy_body("echo");
        functions.set("foo".to_string(), body.clone());

        let retrieved = functions.get("foo");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &body);
    }

    #[test]
    fn test_overwrite() {
        let mut functions = Functions::new();
        let body1 = create_dummy_body("echo1");
        let body2 = create_dummy_body("echo2");

        functions.set("foo".to_string(), body1);
        functions.set("foo".to_string(), body2.clone());

        let retrieved = functions.get("foo");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &body2);
    }

    #[test]
    fn test_get_non_existent() {
        let functions = Functions::new();
        assert!(functions.get("bar").is_none());
    }

    #[test]
    fn test_remove() {
        let mut functions = Functions::new();
        let body = create_dummy_body("echo");
        functions.set("foo".to_string(), body);
        
        assert!(functions.get("foo").is_some());
        functions.remove("foo");
        assert!(functions.get("foo").is_none());
    }
}
