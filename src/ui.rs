use crate::colors::{blue, green};
use crate::path_utils::expand_home;

pub fn format_prompt() -> String {
    let cwd = std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| ".".to_string());
    let cwd_display = expand_home(&cwd);

    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    format!("{}:{}$ ", green(&user), blue(&cwd_display))
}

#[allow(dead_code)]
pub fn format_prompt_with(cwd: &str, user: &str) -> String {
    let cwd_display = expand_home(cwd);
    format!("{}:{}$ ", user, cwd_display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_format_prompt_home() {
        // Use a temporary HOME so expansion to ~ is deterministic
        let tmp_home = TempDir::new().unwrap();
        let original = env::var("HOME").ok();
        env::set_var("HOME", tmp_home.path().to_string_lossy().as_ref());

        let tmp = format!("{}/testdir", env::var("HOME").unwrap());
        let _ = std::fs::create_dir_all(&tmp);

        let p = format_prompt_with(&tmp, "bob");
        assert!(p.contains("~"));
        assert!(p.ends_with("$ "));

        match original {
            Some(v) => env::set_var("HOME", v),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn test_format_prompt_cwd() {
        let tmp = TempDir::new().unwrap();
        let p = format_prompt_with(tmp.path().to_str().unwrap(), "alice");
        assert!(p.contains("alice:"));
    }
}
