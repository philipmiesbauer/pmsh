use std::path::PathBuf;

pub fn expand_home(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        path.replace(&home, "~")
    } else {
        path.to_string()
    }
}

#[allow(dead_code)]
pub fn collapse_tilde(path: &str) -> PathBuf {
    if path.starts_with("~") {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(path.replacen("~", &home, 1))
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home() {
        if let Ok(home) = std::env::var("HOME") {
            let path = format!("{}/test", home);
            let expanded = expand_home(&path);
            assert_eq!(expanded, "~/test");
        }
    }

    #[test]
    fn test_collapse_tilde() {
        if let Ok(home) = std::env::var("HOME") {
            let path = collapse_tilde("~/test");
            assert_eq!(path, PathBuf::from(format!("{}/test", home)));
        }
    }
}
