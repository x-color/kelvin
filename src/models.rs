use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Task state (Phase)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Iced,
    Melting,
    Melted,
    Evaporated,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskState::Iced => "Iced",
            TaskState::Melting => "Melting",
            TaskState::Melted => "Melted",
            TaskState::Evaporated => "Evaporated",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for TaskState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let state = match s {
            "Iced" => TaskState::Iced,
            "Melting" => TaskState::Melting,
            "Melted" => TaskState::Melted,
            "Evaporated" => TaskState::Evaporated,
            _ => anyhow::bail!("Invalid task state: {s}"),
        };
        Ok(state)
    }
}

/// Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub state: TaskState,
    pub thaw_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub created_at: NaiveDate,
    pub note: Option<String>,
}

/// Parses a date specification string, either relative ("3d", "1w") or absolute ("2026-03-01"), into a NaiveDate.
pub fn parse_date_spec(spec: &str, base: NaiveDate) -> anyhow::Result<NaiveDate> {
    // Relative date: Number + 'd' or 'w'
    if let Some(num_str) = spec.strip_suffix('d') {
        let days: i64 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid relative date format: {spec}"))?;
        return base
            .checked_add_days(chrono::Days::new(days as u64))
            .ok_or_else(|| anyhow::anyhow!("Date overflow"));
    }
    if let Some(num_str) = spec.strip_suffix('w') {
        let weeks: i64 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid relative date format: {spec}"))?;
        return base
            .checked_add_days(chrono::Days::new((weeks * 7) as u64))
            .ok_or_else(|| anyhow::anyhow!("Date overflow"));
    }
    // Absolute date: YYYY-MM-DD
    NaiveDate::parse_from_str(spec, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Invalid date format '{spec}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_relative_days() {
        let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let result = parse_date_spec("3d", base).unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2026, 1, 4).unwrap());
    }

    #[test]
    fn parse_relative_weeks() {
        let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let result = parse_date_spec("2w", base).unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
    }

    #[test]
    fn parse_absolute_date() {
        let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let result = parse_date_spec("2026-03-15", base).unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
    }

    #[test]
    fn parse_invalid_format() {
        let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(parse_date_spec("abc", base).is_err());
        assert!(parse_date_spec("3x", base).is_err());
    }

    #[test]
    fn task_state_display() {
        assert_eq!(format!("{}", TaskState::Iced), "Iced");
        assert_eq!(format!("{}", TaskState::Melting), "Melting");
        assert_eq!(format!("{}", TaskState::Melted), "Melted");
        assert_eq!(format!("{}", TaskState::Evaporated), "Evaporated");
    }

}
