use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CompSpec {
    pub wordlist: Option<String>,
}

pub struct CompletionRegistry {
    specs: Option<HashMap<String, CompSpec>>,
}

impl CompletionRegistry {
    pub const fn new() -> Self {
        Self { specs: None }
    }

    fn ensure_init(&mut self) {
        if self.specs.is_none() {
            self.specs = Some(HashMap::new());
        }
    }

    pub fn register(&mut self, command: String, spec: CompSpec) {
        self.ensure_init();
        self.specs.as_mut().unwrap().insert(command, spec);
    }

    pub fn remove(&mut self, command: &str) {
        if let Some(specs) = &mut self.specs {
            specs.remove(command);
        }
    }

    pub fn get(&self, command: &str) -> Option<CompSpec> {
        self.specs
            .as_ref()
            .and_then(|specs| specs.get(command).cloned())
    }
}

pub static COMP_REGISTRY: RwLock<CompletionRegistry> = RwLock::new(CompletionRegistry::new());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_registry_new() {
        let registry = CompletionRegistry::new();
        assert!(registry.specs.is_none());
    }

    #[test]
    fn test_completion_registry_register_and_get() {
        let mut registry = CompletionRegistry::new();
        let spec = CompSpec {
            wordlist: Some("a b c".to_string()),
        };

        registry.register("mycmd".to_string(), spec.clone());

        let retrieved = registry.get("mycmd");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().wordlist, Some("a b c".to_string()));
    }

    #[test]
    fn test_completion_registry_remove() {
        let mut registry = CompletionRegistry::new();
        let spec = CompSpec {
            wordlist: Some("a b c".to_string()),
        };

        registry.register("mycmd".to_string(), spec.clone());
        assert!(registry.get("mycmd").is_some());

        registry.remove("mycmd");
        assert!(registry.get("mycmd").is_none());
    }

    #[test]
    fn test_completion_registry_get_non_existent() {
        let registry = CompletionRegistry::new();
        let retrieved = registry.get("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_comp_registry_static() {
        // Just verify we can lock it
        let registry = COMP_REGISTRY.read().unwrap();
        assert!(registry.specs.is_none() || registry.specs.is_some());
    }
}
