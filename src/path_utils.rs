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
        use serial_test::serial;

        #[serial]
        fn inner() {
            // Set HOME to a known temp directory to make this test deterministic
            let tmp = tempfile::TempDir::new().unwrap();
            let home_guard = {
                let original = std::env::var("HOME").ok();
                std::env::set_var("HOME", tmp.path().to_string_lossy().as_ref());
                // guard to restore
                struct G(Option<String>);
                impl Drop for G {
                    fn drop(&mut self) {
                        match &self.0 {
                            Some(v) => std::env::set_var("HOME", v),
                            None => std::env::remove_var("HOME"),
                        }
                    }
                }
                G(original)
            };

            let home = std::env::var("HOME").unwrap();
            let path = format!("{}/test", home);
            let expanded = expand_home(&path);
            assert!(expanded.starts_with("~"));
            drop(home_guard);
        }

        inner();
    }

    #[test]
    fn test_collapse_tilde() {
        use serial_test::serial;

        #[serial]
        fn inner() {
            // Set HOME to a known temp directory
            let tmp = tempfile::TempDir::new().unwrap();
            let original = std::env::var("HOME").ok();
            std::env::set_var("HOME", tmp.path().to_string_lossy().as_ref());

            let home = std::env::var("HOME").unwrap();
            let path = collapse_tilde("~/test");
            assert_eq!(path, PathBuf::from(format!("{}/test", home)));

            match original {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }

        inner();
    }
}
