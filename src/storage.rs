use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::models::Task;

/// Task storage using a local JSON file
pub struct TaskStore {
    path: PathBuf,
}

impl TaskStore {
    /// Create a store with a path based on the configuration
    pub fn from_config(config: &Config) -> Result<Self> {
        let path = config.data_file_path()?;
        Ok(Self { path })
    }

    /// Create a store with a specific path (for testing)
    #[cfg(test)]
    pub fn new_with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load the task list. Returns an empty Vec if the file does not exist.
    pub fn load(&self) -> Result<Vec<Task>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {}", self.path.display()))?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        let tasks: Vec<Task> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", self.path.display()))?;
        Ok(tasks)
    }

    /// Save the task list
    pub fn save(&self, tasks: &[Task]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let content = serde_json::to_string_pretty(tasks)?;
        fs::write(&self.path, content)
            .with_context(|| format!("Failed to write {}", self.path.display()))?;
        Ok(())
    }

    /// Get the next ID (existing maximum ID + 1, or 1 if none exist)
    pub fn next_id(tasks: &[Task]) -> u32 {
        tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskState, Task};
    use chrono::NaiveDate;

    fn sample_task(id: u32) -> Task {
        Task {
            id,
            title: format!("Task {id}"),
            description: String::new(),
            state: TaskState::Melted,
            thaw_date: None,
            due_date: None,
            created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        }
    }

    #[test]
    fn load_nonexistent_file() {
        let store = TaskStore::new_with_path(PathBuf::from("/tmp/kelvin_test_nonexistent.json"));
        let tasks = store.load().unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        let store = TaskStore::new_with_path(path.clone());

        let tasks = vec![sample_task(1), sample_task(2)];
        store.save(&tasks).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, 1);
        assert_eq!(loaded[1].id, 2);
    }

    #[test]
    fn next_id_empty() {
        assert_eq!(TaskStore::next_id(&[]), 1);
    }

    #[test]
    fn next_id_with_tasks() {
        let tasks = vec![sample_task(5), sample_task(3)];
        assert_eq!(TaskStore::next_id(&tasks), 6);
    }
}
