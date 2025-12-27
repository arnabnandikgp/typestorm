use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub timestamp: DateTime<Local>,
    pub mode: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub wpm_history: Vec<(f64, f64)>,
    pub error_points: Vec<(f64, f64)>,
}

pub fn get_history_file_path() -> Result<PathBuf> {
    #[cfg(test)]
    return Ok(std::env::temp_dir().join("typestorm_test_history.json"));

    #[cfg(not(test))]
    {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".typestorm_history.json"))
    }
}

pub fn load_history() -> Result<Vec<TestResult>> {
    let path = get_history_file_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let history: Vec<TestResult> = serde_json::from_str(&content)?;
    Ok(history)
}

pub fn save_history(history: &[TestResult]) -> Result<()> {
    let path = get_history_file_path()?;
    let content = serde_json::to_string_pretty(history)?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_persistence() {
        let result = TestResult {
            timestamp: Local::now(),
            mode: "Words: 10".to_string(),
            wpm: 60.0,
            accuracy: 98.5,
            wpm_history: vec![(1.0, 50.0), (2.0, 60.0)],
            error_points: vec![(1.5, 55.0)],
        };

        let history = vec![result.clone()];
        save_history(&history).expect("Failed to save history");

        let loaded = load_history().expect("Failed to load history");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].mode, "Words: 10");
        assert_eq!(loaded[0].wpm, 60.0);
        assert_eq!(loaded[0].wpm_history.len(), 2);
    }
}
