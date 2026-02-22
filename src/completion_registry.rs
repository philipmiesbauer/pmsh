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
