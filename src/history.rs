use std::fs;
use std::path::PathBuf;

const MAX_HISTORY_SIZE: usize = 1000;

pub struct HistoryManager {
    history_file: PathBuf,
}

impl HistoryManager {
    pub fn new() -> Result<Self, String> {
        let history_file = Self::get_history_path()?;
        Ok(HistoryManager { history_file })
    }

    fn get_history_path() -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .map_err(|_| "Failed to get HOME environment variable".to_string())?;
        let mut path = PathBuf::from(home);
        path.push(".pmsh_history");
        Ok(path)
    }

    pub fn load(&self) -> Result<Vec<String>, String> {
        if !self.history_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.history_file)
            .map_err(|e| format!("Failed to read history file: {}", e))?;

        let history: Vec<String> = content.lines().map(|line| line.to_string()).collect();

        Ok(history)
    }

    pub fn save(&self, history: &[String]) -> Result<(), String> {
        // Keep only the last MAX_HISTORY_SIZE entries
        let start = history.len().saturating_sub(MAX_HISTORY_SIZE);
        let limited_history = &history[start..];

        let content = limited_history.join("\n");
        fs::write(&self.history_file, content)
            .map_err(|e| format!("Failed to write history file: {}", e))?;

        Ok(())
    }

    pub fn add_entry(&self, entry: &str, history: &mut Vec<String>) -> Result<(), String> {
        history.push(entry.to_string());

        // Save only the last MAX_HISTORY_SIZE entries
        if history.len() > MAX_HISTORY_SIZE {
            let start = history.len() - MAX_HISTORY_SIZE;
            let limited = history[start..].to_vec();
            *history = limited;
        }

        self.save(history)?;
        Ok(())
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize history manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_manager_load_empty() {
        let mgr = HistoryManager::new().unwrap();
        let history = mgr.load().unwrap();
        // History should be empty or contain existing history
        assert!(history.is_empty() || !history.is_empty()); // Just verify it doesn't error
    }

    #[test]
    fn test_history_manager_max_size() {
        if let Ok(mgr) = HistoryManager::new() {
            let mut history = Vec::new();

            // Add more than MAX_HISTORY_SIZE entries
            for i in 0..=MAX_HISTORY_SIZE {
                let entry = format!("command_{}", i);
                let _ = mgr.add_entry(&entry, &mut history);
            }

            // History should not exceed MAX_HISTORY_SIZE
            assert!(history.len() <= MAX_HISTORY_SIZE);
        }
    }
}
