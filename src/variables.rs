use std::collections::{HashMap, HashSet};
use std::env;

#[derive(Debug, Clone, Default)]
pub struct Variables {
    vars: HashMap<String, String>,
    exported: HashSet<String>,
    positional_args: Vec<String>,
}

impl Variables {
    pub fn new() -> Self {
        let mut vars = HashMap::new();
        let mut exported = HashSet::new();
        // Initialize with environment variables — all are considered exported
        for (key, value) in env::vars() {
            exported.insert(key.clone());
            vars.insert(key, value);
        }
        Variables {
            vars,
            exported,
            positional_args: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }

    /// Remove a variable from the shell's internal map only.
    /// Used for scoped variable restoration (e.g. temporary VAR=val assignments).
    /// Does NOT touch the process environment.
    pub fn remove(&mut self, key: &str) {
        self.vars.remove(key);
        self.exported.remove(key);
    }

    /// Mark a variable as exported to child processes.
    /// If the variable exists in the shell, it is also propagated to the process environment.
    pub fn export(&mut self, key: &str) {
        self.exported.insert(key.to_string());
        if let Some(val) = self.vars.get(key) {
            env::set_var(key, val);
        }
    }

    /// Unset a variable: removes it from the shell and the process environment.
    pub fn unset(&mut self, key: &str) {
        self.vars.remove(key);
        self.exported.remove(key);
        env::remove_var(key);
    }

    /// Returns true if the given variable is currently marked as exported.
    #[allow(dead_code)]
    pub fn is_exported(&self, key: &str) -> bool {
        self.exported.contains(key)
    }

    /// Returns an iterator over exported variable names and their values.
    pub fn exported_vars(&self) -> impl Iterator<Item = (&str, &str)> {
        self.exported
            .iter()
            .filter_map(move |k| self.vars.get(k).map(|v| (k.as_str(), v.as_str())))
    }

    pub fn set_positional_args(&mut self, args: Vec<String>) {
        self.positional_args = args;
    }

    pub fn get_positional_args(&self) -> Vec<String> {
        self.positional_args.clone()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        if let Ok(idx) = key.parse::<usize>() {
            if idx > 0 && idx <= self.positional_args.len() {
                return Some(&self.positional_args[idx - 1]);
            }
            // $0 is usually the shell name or script name, not handled in positional_args yet
            // but we can return None or handle it if we store it.
            return None;
        }
        self.vars.get(key)
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        self.vars.clone()
    }

    /// Expand variables in a string.
    /// Replaces $VAR with its value.
    pub fn expand(&self, input: &str) -> String {
        if !input.contains('$') {
            return input.to_string();
        }

        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                let mut var_name = String::new();

                // Handle braced variables like ${VAR}
                // Check for special single-character vars first
                if let Some(&next_char) = chars.peek() {
                    if matches!(next_char, '@' | '*' | '#' | '?' | '-' | '$' | '!') {
                        var_name.push(chars.next().unwrap());
                    } else if next_char.is_ascii_digit() {
                        // Only single digit for now unless braced (but braced is not handled here yet)
                        // Actually bash supports $10 but usually parsed as $1 then 0.
                        // But if we parse digits...
                        // Let's just consume one digit for simple expansion
                        var_name.push(chars.next().unwrap());
                    } else {
                        while let Some(&next_char) = chars.peek() {
                            if next_char.is_alphanumeric() || next_char == '_' {
                                var_name.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                    }
                }

                if var_name.is_empty() {
                    result.push('$');
                } else if var_name == "@" || var_name == "*" {
                    result.push_str(&self.positional_args.join(" "));
                } else if var_name == "#" {
                    result.push_str(&self.positional_args.len().to_string());
                } else if var_name == "$" {
                    result.push_str(&std::process::id().to_string());
                } else if let Some(val) = self.get(&var_name) {
                    result.push_str(val);
                }
                // If var not found, it expands to empty string
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

        vars.set_positional_args(vec!["arg1".to_string(), "arg2".to_string()]);
        assert_eq!(vars.expand("$1"), "arg1");
        assert_eq!(vars.expand("$2"), "arg2");
        assert_eq!(vars.expand("$3"), ""); // Non-existent positional arg

        assert_eq!(vars.expand("$*"), "arg1 arg2");

        vars.set_positional_args(vec!["single".to_string()]);
        assert_eq!(vars.expand("$@"), "single");
    }

    #[test]
    fn test_variable_special_vars() {
        let mut vars = Variables::new();
        // $$ is process ID
        // We can just verify it expands to something non-empty and changes based on std::process::id
        let pid = std::process::id().to_string();
        assert_eq!(vars.expand("$$"), pid);

        // $! is not implemented
        assert_eq!(vars.expand("$!"), "");

        // $? exit status
        vars.set("?".to_string(), "1".to_string());
        assert_eq!(vars.expand("$?"), "1");

        // $- is not implemented (expands to empty normally without set)
        assert_eq!(vars.expand("$-"), "");

        // $# number of arguments
        vars.set_positional_args(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(vars.expand("$#"), "2");
    }

    #[test]
    fn test_variable_remove() {
        let mut vars = Variables::new();
        vars.set("TEST_VAR".to_string(), "value".to_string());
        assert_eq!(vars.get("TEST_VAR"), Some(&"value".to_string()));

        vars.remove("TEST_VAR");
        assert_eq!(vars.get("TEST_VAR"), None);
    }

    #[test]
    fn test_to_env_vars() {
        let mut vars = Variables::new();
        vars.set("A".to_string(), "1".to_string());
        vars.set("B".to_string(), "2".to_string());

        let env_map = vars.to_env_vars();
        assert!(env_map.contains_key("A"));
        assert!(env_map.contains_key("B"));
        assert_eq!(env_map.get("A").unwrap(), "1");
        assert_eq!(env_map.get("B").unwrap(), "2");

        // Internal variables shouldn't leak
        assert!(!env_map.contains_key("?"));
    }
}
