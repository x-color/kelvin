use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use rusqlite::{params, Connection};

use crate::config::Config;
use crate::models::{Task, TaskState};

/// Task storage using a SQLite database
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

    fn init_db(&self) -> Result<Connection> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let conn = Connection::open(&self.path)
            .with_context(|| format!("Failed to open database {}", self.path.display()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                state TEXT NOT NULL,
                thaw_date TEXT,
                due_date TEXT,
                created_at TEXT NOT NULL,
                note TEXT
            )",
            [],
        )
        .with_context(|| "Failed to initialize tasks table schema")?;

        Ok(conn)
    }

    #[cfg(test)]
    pub fn load(&self) -> Result<Vec<Task>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let conn = self.init_db()?;
        Self::load_from_conn(&conn)
    }

    fn load_from_conn(conn: &Connection) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare("SELECT id, title, description, state, thaw_date, due_date, created_at, note FROM tasks ORDER BY id ASC")?;

        let task_iter = stmt.query_map([], |row| {
            let id: u32 = row.get(0)?;
            let title: String = row.get(1)?;
            let description: String = row.get(2)?;
            let state_str: String = row.get(3)?;
            let thaw_date_str: Option<String> = row.get(4)?;
            let due_date_str: Option<String> = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let note: Option<String> = row.get(7)?;

            let state = TaskState::from_str(&state_str).unwrap_or(TaskState::Melted);
            let thaw_date =
                thaw_date_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
            let due_date =
                due_date_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
            let created_at =
                NaiveDate::parse_from_str(&created_at_str, "%Y-%m-%d").unwrap_or_default();

            Ok(Task {
                id,
                title,
                description,
                state,
                thaw_date,
                due_date,
                created_at,
                note,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }

        Ok(tasks)
    }

    /// Update tasks using a closure within an exclusive transaction
    pub fn update<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Vec<Task>) -> Result<R>,
    {
        let mut conn = self.init_db()?;

        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)?;

        // 1. Load current tasks
        let mut tasks = Self::load_from_conn(&tx)?;

        // 2. Perform the update via the closure
        let result = f(&mut tasks)?;

        // 3. Delete existing tasks and re-insert the new state
        tx.execute("DELETE FROM tasks", [])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO tasks (id, title, description, state, thaw_date, due_date, created_at, note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            )?;
            for task in &tasks {
                let thaw_date_str = task.thaw_date.map(|d| d.format("%Y-%m-%d").to_string());
                let due_date_str = task.due_date.map(|d| d.format("%Y-%m-%d").to_string());
                let created_at_str = task.created_at.format("%Y-%m-%d").to_string();

                stmt.execute(params![
                    task.id,
                    task.title,
                    task.description,
                    task.state.to_string(),
                    thaw_date_str,
                    due_date_str,
                    created_at_str,
                    task.note
                ])?;
            }
        }

        tx.commit()?;

        Ok(result)
    }

    /// Save the task list (used by tests)
    #[cfg(test)]
    pub fn save(&self, tasks: &[Task]) -> Result<()> {
        let mut conn = self.init_db()?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM tasks", [])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO tasks (id, title, description, state, thaw_date, due_date, created_at, note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            )?;
            for task in tasks {
                let thaw_date_str = task.thaw_date.map(|d| d.format("%Y-%m-%d").to_string());
                let due_date_str = task.due_date.map(|d| d.format("%Y-%m-%d").to_string());
                let created_at_str = task.created_at.format("%Y-%m-%d").to_string();

                stmt.execute(params![
                    task.id,
                    task.title,
                    task.description,
                    task.state.to_string(),
                    thaw_date_str,
                    due_date_str,
                    created_at_str,
                    task.note
                ])?;
            }
        }
        tx.commit()?;
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
    use crate::models::{Task, TaskState};
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
            note: None,
        }
    }

    #[test]
    fn load_nonexistent_file() {
        let store = TaskStore::new_with_path(PathBuf::from("/tmp/kelvin_test_nonexistent.db"));
        let tasks = store.load().unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.db");
        let store = TaskStore::new_with_path(path.clone());

        let tasks = vec![sample_task(1), sample_task(2)];
        store.save(&tasks).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, 1);
        assert_eq!(loaded[1].id, 2);
    }

    #[test]
    fn update_transaction() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.db");
        let store = TaskStore::new_with_path(path.clone());

        store
            .update(|tasks| {
                tasks.push(sample_task(1));
                tasks.push(sample_task(2));
                Ok(())
            })
            .unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 2);

        store
            .update(|tasks| {
                tasks.remove(0);
                Ok(())
            })
            .unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, 2);
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
