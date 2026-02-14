use clap::{Parser, Subcommand};

/// Kelvin - A CLI task management tool using thermodynamic metaphors
#[derive(Parser, Debug)]
#[command(name = "kelvin", version, about = "A thermodynamic task manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new task
    Add {
        /// Task title
        title: String,
        /// Task description
        #[arg(long = "desc")]
        description: Option<String>,
        /// Thaw date (e.g., 3d, 1w, 2026-03-01). If specified, the task is created in Iced state.
        #[arg(short = 'd', long = "date")]
        thaw_date: Option<String>,
        /// Due date (e.g., 3d, 1w, 2026-03-01)
        #[arg(long = "due")]
        due_date: Option<String>,
    },

    /// Edit an existing task
    Edit {
        /// Task ID
        id: u32,
        /// New title
        #[arg(short = 't', long = "title")]
        title: Option<String>,
        /// New description
        #[arg(long = "desc")]
        description: Option<String>,
        /// Change the thaw date (e.g., 3d, 1w, 2026-03-01)
        #[arg(short = 'd', long = "date")]
        thaw_date: Option<String>,
        /// Change the due date (e.g., 3d, 1w, 2026-03-01)
        #[arg(long = "due")]
        due_date: Option<String>,
    },

    /// Show task details
    Show {
        /// Task ID
        id: u32,
    },

    /// List tasks
    List {
        /// Show frozen (Iced) tasks
        #[arg(long)]
        iced: bool,
        /// Show all tasks
        #[arg(long)]
        all: bool,
    },

    /// Set task to ready state (Melting/Iced -> Melted)
    Warm {
        /// Task ID
        id: u32,
    },

    /// Complete (evaporate) a task (Melted/Iced -> Evaporated)
    Burn {
        /// Task ID
        id: u32,
    },

    /// Cancel completion (Evaporated -> Melted)
    Cool {
        /// Task ID
        id: u32,
    },

    /// Refreeze a task (-> Iced)
    Freeze {
        /// Task ID
        id: u32,
        /// Thaw date (e.g., 3d, 1w, 2026-03-01)
        #[arg(short = 'd', long = "date")]
        thaw_date: Option<String>,
    },
}
