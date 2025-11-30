use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Default)]
pub struct Variables {
    vars: HashMap<String, String>,
}

impl Variables {
    pub fn new() -> Self {
        let mut vars = HashMap::new();
        // Initialize with environment variables
        // Note: This copies all environment variables, including potentially sensitive ones
        // (e.g. SSH_AUTH_SOCK, AWS_SECRET_ACCESS_KEY).
        // This is standard shell behavior but means they are accessible for expansion.
        for (key, value) in env::vars() {
            vars.insert(key, value);
        }
        Variables { vars }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        self.vars.clone()
    }

    /// Expand variables in a string.
    /// Replaces $VAR with its value.
    /// Simple implementation: splits by $ and looks up keys.
    /// Note: This is a basic implementation and doesn't handle complex cases like ${VAR} or escaping yet.
    pub fn expand(&self, input: &str) -> String {
        if !input.contains('$') {
            return input.to_string();
        }

        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                let mut var_name = String::new();
                while let Some(&next_char) = chars.peek() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if var_name.is_empty() {
                    result.push('$');
                } else if let Some(val) = self.get(&var_name) {
                    result.push_str(val);
                }
                // If var not found, it expands to empty string (standard shell behavior)
            } else {
                result.push(c);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_expansion() {
        let mut vars = Variables::new();
        vars.set("FOO".to_string(), "bar".to_string());
        vars.set("BAZ".to_string(), "qux".to_string());

        assert_eq!(vars.expand("hello $FOO"), "hello bar");
        assert_eq!(vars.expand("$FOO world"), "bar world");
        assert_eq!(vars.expand("$FOO$BAZ"), "barqux");
        assert_eq!(vars.expand("no vars"), "no vars");
        assert_eq!(vars.expand("$NONEXISTENT"), "");
        assert_eq!(vars.expand("$"), "$");
    }
}
