use anyhow::{bail, Result};
use chrono::NaiveDate;

use crate::models::{Task, TaskState};

/// Automatically transition Iced tasks that have passed their thaw date to the Melting state during command execution.
/// Returns the number of tasks that were updated.
pub fn auto_warm(tasks: &mut [Task], today: NaiveDate) -> u32 {
    let mut count = 0;
    for task in tasks.iter_mut() {
        if task.state == TaskState::Iced {
            if let Some(thaw_date) = task.thaw_date {
                if today >= thaw_date {
                    task.state = TaskState::Melting;
                    count += 1;
                }
            }
        }
    }
    count
}

/// Melting/Iced -> Melted: Set the task to a ready (Melted) state.
pub fn warm(task: &mut Task) -> Result<()> {
    match task.state {
        TaskState::Melting | TaskState::Iced => {
            task.state = TaskState::Melted;
            task.thaw_date = None;
            Ok(())
        }
        _ => bail!(
            "Cannot warm task {} (state: {}). Only Iced or Melting tasks can be warmed.",
            task.id,
            task.state
        ),
    }
}

/// Melted/Iced -> Evaporated: Complete (evaporate) the task.
pub fn burn(task: &mut Task) -> Result<()> {
    match task.state {
        TaskState::Melted | TaskState::Iced => {
            task.state = TaskState::Evaporated;
            Ok(())
        }
        _ => bail!(
            "Cannot burn task {} (state: {}). Only Melted or Iced tasks can be burned.",
            task.id,
            task.state
        ),
    }
}

/// Evaporated -> Melted: Cancel completion and return the task to a Melted state.
pub fn cool(task: &mut Task) -> Result<()> {
    match task.state {
        TaskState::Evaporated => {
            task.state = TaskState::Melted;
            task.thaw_date = None;
            Ok(())
        }
        _ => bail!(
            "Cannot cool task {} (state: {}). Only Evaporated tasks can be cooled.",
            task.id,
            task.state
        ),
    }
}

/// Any State -> Iced: Refreeze the task. A thaw date is required.
pub fn freeze(task: &mut Task, thaw_date: NaiveDate) -> Result<()> {
    task.state = TaskState::Iced;
    task.thaw_date = Some(thaw_date);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;

    fn make_task(state: TaskState, thaw_date: Option<NaiveDate>) -> Task {
        Task {
            id: 1,
            title: "Test".to_string(),
            description: String::new(),
            state,
            thaw_date,
            due_date: None,
            created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        }
    }

    // --- auto_warm ---
    #[test]
    fn auto_warm_transitions_iced_past_thaw_date() {
        let mut tasks = vec![make_task(
            TaskState::Iced,
            Some(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()),
        )];
        let today = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        let count = auto_warm(&mut tasks, today);
        assert_eq!(count, 1);
        assert_eq!(tasks[0].state, TaskState::Melting);
    }

    #[test]
    fn auto_warm_ignores_iced_before_thaw_date() {
        let mut tasks = vec![make_task(
            TaskState::Iced,
            Some(NaiveDate::from_ymd_opt(2026, 1, 10).unwrap()),
        )];
        let today = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        let count = auto_warm(&mut tasks, today);
        assert_eq!(count, 0);
        assert_eq!(tasks[0].state, TaskState::Iced);
    }

    #[test]
    fn auto_warm_ignores_non_iced() {
        let mut tasks = vec![make_task(TaskState::Melted, None)];
        let today = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        let count = auto_warm(&mut tasks, today);
        assert_eq!(count, 0);
        assert_eq!(tasks[0].state, TaskState::Melted);
    }

    // --- warm ---
    #[test]
    fn warm_melting_to_melted() {
        let mut task = make_task(TaskState::Melting, None);
        warm(&mut task).unwrap();
        assert_eq!(task.state, TaskState::Melted);
    }

    #[test]
    fn warm_iced_to_melted() {
        let mut task = make_task(
            TaskState::Iced,
            Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        );
        warm(&mut task).unwrap();
        assert_eq!(task.state, TaskState::Melted);
        assert_eq!(task.thaw_date, None);
    }

    #[test]
    fn warm_melted_fails() {
        let mut task = make_task(TaskState::Melted, None);
        assert!(warm(&mut task).is_err());
    }

    #[test]
    fn warm_evaporated_fails() {
        let mut task = make_task(TaskState::Evaporated, None);
        assert!(warm(&mut task).is_err());
    }

    // --- burn ---
    #[test]
    fn burn_melted_to_evaporated() {
        let mut task = make_task(TaskState::Melted, None);
        burn(&mut task).unwrap();
        assert_eq!(task.state, TaskState::Evaporated);
    }

    #[test]
    fn burn_iced_to_evaporated() {
        let mut task = make_task(
            TaskState::Iced,
            Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        );
        burn(&mut task).unwrap();
        assert_eq!(task.state, TaskState::Evaporated);
    }

    #[test]
    fn burn_evaporated_fails() {
        let mut task = make_task(TaskState::Evaporated, None);
        assert!(burn(&mut task).is_err());
    }

    // --- cool ---
    #[test]
    fn cool_evaporated_to_melted() {
        let mut task = make_task(
            TaskState::Evaporated,
            Some(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()),
        );
        cool(&mut task).unwrap();
        assert_eq!(task.state, TaskState::Melted);
        assert_eq!(task.thaw_date, None);
    }

    #[test]
    fn cool_melted_fails() {
        let mut task = make_task(TaskState::Melted, None);
        assert!(cool(&mut task).is_err());
    }

    // --- freeze ---
    #[test]
    fn freeze_melted_to_iced() {
        let mut task = make_task(TaskState::Melted, None);
        let date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        freeze(&mut task, date).unwrap();
        assert_eq!(task.state, TaskState::Iced);
        assert_eq!(task.thaw_date, Some(date));
    }

    #[test]
    fn freeze_evaporated_to_iced() {
        let mut task = make_task(TaskState::Evaporated, None);
        let date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        freeze(&mut task, date).unwrap();
        assert_eq!(task.state, TaskState::Iced);
        assert_eq!(task.thaw_date, Some(date));
    }
}
