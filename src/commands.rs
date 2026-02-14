use anyhow::Result;
use chrono::Local;
use colored::Colorize;

use crate::cli::Commands;
use crate::config::Config;
use crate::models::{parse_date_spec, Task, TaskState};
use crate::state;
use crate::storage::TaskStore;

/// 状態に応じた色付き文字列を返す
fn colored_state(state: TaskState) -> String {
    let label = state.to_string();
    match state {
        TaskState::Iced => label.truecolor(0xBB, 0xE8, 0xF2).to_string(),
        TaskState::Melting => label.truecolor(0x94, 0xD7, 0xF2).to_string(),
        TaskState::Melted => label.truecolor(0x55, 0xB3, 0xD9).to_string(),
        TaskState::Evaporated => label.truecolor(0x3F, 0x5F, 0x73).to_string(),
    }
}

/// 色付き文字列を指定幅にパディングする (ANSI codes の外側にスペースを追加)
fn colored_state_padded(state: TaskState, width: usize) -> String {
    let visible_len = state.to_string().len();
    let colored = colored_state(state);
    let padding = width.saturating_sub(visible_len);
    format!("{colored}{}", " ".repeat(padding))
}

/// 日付を文字列に変換 (None は "-")
fn date_str(date: Option<chrono::NaiveDate>) -> String {
    date.map(|d| d.to_string()).unwrap_or_else(|| "-".to_string())
}

/// コマンド実行のメインディスパッチャ
pub fn execute(command: Commands) -> Result<()> {
    let config = Config::load()?;
    let store = TaskStore::from_config(&config)?;
    let today = Local::now().date_naive();

    match command {
        Commands::Add {
            title,
            description,
            thaw_date,
            due_date,
        } => cmd_add(
            &store,
            &title,
            description.as_deref(),
            thaw_date.as_deref(),
            due_date.as_deref(),
            today,
        )?,
        Commands::Edit {
            id,
            title,
            description,
            thaw_date,
            due_date,
        } => cmd_edit(
            &store,
            id,
            title.as_deref(),
            description.as_deref(),
            thaw_date.as_deref(),
            due_date.as_deref(),
            today,
        )?,
        Commands::Show { id } => cmd_show(&store, id, today)?,
        Commands::List { iced, all } => cmd_list(&store, iced, all, today)?,
        Commands::Warm { id } => cmd_warm(&store, id, today)?,
        Commands::Burn { id } => cmd_burn(&store, id, today)?,
        Commands::Cool { id } => cmd_cool(&store, id, today)?,
        Commands::Freeze { id, thaw_date } => {
            cmd_freeze(&store, id, thaw_date.as_deref(), today, &config)?
        }
    }

    Ok(())
}

/// タスクを追加する
fn cmd_add(
    store: &TaskStore,
    title: &str,
    description: Option<&str>,
    thaw_date_spec: Option<&str>,
    due_date_spec: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<()> {
    let mut tasks = store.load()?;
    let id = TaskStore::next_id(&tasks);

    let (task_state, thaw_date) = match thaw_date_spec {
        Some(spec) => {
            let date = parse_date_spec(spec, today)?;
            (TaskState::Iced, Some(date))
        }
        None => (TaskState::Melted, None),
    };

    let due_date = match due_date_spec {
        Some(spec) => Some(parse_date_spec(spec, today)?),
        None => None,
    };

    let task = Task {
        id,
        title: title.to_string(),
        description: description.unwrap_or_default().to_string(),
        state: task_state,
        thaw_date,
        due_date,
        created_at: today,
    };

    println!(
        "Added task {} [{}]: {}",
        task.id,
        task.state,
        task.title
    );

    tasks.push(task);
    store.save(&tasks)?;
    Ok(())
}

/// タスクを編集する
fn cmd_edit(
    store: &TaskStore,
    id: u32,
    new_title: Option<&str>,
    new_description: Option<&str>,
    new_thaw_date: Option<&str>,
    new_due_date: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<()> {
    let mut tasks = store.load()?;
    state::auto_warm(&mut tasks, today);

    let task = tasks
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("Task {id} not found"))?;

    if let Some(title) = new_title {
        task.title = title.to_string();
    }
    if let Some(desc) = new_description {
        task.description = desc.to_string();
    }
    if let Some(spec) = new_thaw_date {
        task.thaw_date = Some(parse_date_spec(spec, today)?);
    }
    if let Some(spec) = new_due_date {
        task.due_date = Some(parse_date_spec(spec, today)?);
    }

    println!(
        "Updated task {} [{}]: {}",
        task.id,
        task.state,
        task.title
    );

    store.save(&tasks)?;
    Ok(())
}

/// タスクの詳細を表示する
fn cmd_show(store: &TaskStore, id: u32, today: chrono::NaiveDate) -> Result<()> {
    let mut tasks = store.load()?;
    state::auto_warm(&mut tasks, today);
    store.save(&tasks)?;

    let task = tasks
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("Task {id} not found"))?;

    println!("{:<14} {}", "ID:".bold(), task.id);
    println!("{:<14} {}", "Title:".bold(), task.title);
    if !task.description.is_empty() {
        println!("{:<14} {}", "Description:".bold(), task.description);
    }
    println!("{:<14} {}", "State:".bold(), colored_state(task.state));
    println!("{:<14} {}", "Thaw Date:".bold(), date_str(task.thaw_date));
    println!("{:<14} {}", "Due Date:".bold(), date_str(task.due_date));
    println!("{:<14} {}", "Created:".bold(), task.created_at);

    Ok(())
}

/// タスク一覧を表示する
/// カラム順: ID, Task, State, Thaw Date, Due Date
fn cmd_list(store: &TaskStore, iced: bool, all: bool, today: chrono::NaiveDate) -> Result<()> {
    let mut tasks = store.load()?;
    let warmed = state::auto_warm(&mut tasks, today);
    if warmed > 0 {
        store.save(&tasks)?;
    }

    let filtered: Vec<&Task> = if all {
        tasks.iter().collect()
    } else if iced {
        tasks
            .iter()
            .filter(|t| t.state == TaskState::Iced)
            .collect()
    } else {
        // デフォルト: Melting と Melted のみ
        tasks
            .iter()
            .filter(|t| t.state == TaskState::Melting || t.state == TaskState::Melted)
            .collect()
    };

    if filtered.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    // カラム幅の定義
    let id_w = 5;
    let task_w = filtered
        .iter()
        .map(|t| t.title.len())
        .max()
        .unwrap_or(4)
        .max(4); // "Task" の長さ以上
    let state_w = 11; // "Evaporated" = 10 + 余白
    let date_w = 12; // "YYYY-MM-DD" = 10 + 余白

    // ヘッダー (bold もANSIコードを含むため、手動パディング)
    println!(
        "{}  {}  {}  {}  {}",
        format!("{:<id_w$}", "ID").bold(),
        format!("{:<task_w$}", "Task").bold(),
        format!("{:<state_w$}", "State").bold(),
        format!("{:<date_w$}", "Thaw Date").bold(),
        "Due Date".bold(),
    );
    let total_w = id_w + 2 + task_w + 2 + state_w + 2 + date_w + 2 + date_w;
    println!("{}", "─".repeat(total_w));

    for task in &filtered {
        println!(
            "{:<id_w$}  {:<task_w$}  {}  {:<date_w$}  {}",
            task.id,
            task.title,
            colored_state_padded(task.state, state_w),
            date_str(task.thaw_date),
            date_str(task.due_date),
        );
    }

    Ok(())
}

/// Melting/Iced → Melted
fn cmd_warm(store: &TaskStore, id: u32, today: chrono::NaiveDate) -> Result<()> {
    let mut tasks = store.load()?;
    state::auto_warm(&mut tasks, today);

    let task = tasks
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("Task {id} not found"))?;

    state::warm(task)?;
    println!(
        "Warmed task {} [{}]: {}",
        task.id, task.state, task.title
    );

    store.save(&tasks)?;
    Ok(())
}

/// Melted/Iced → Evaporated
fn cmd_burn(store: &TaskStore, id: u32, today: chrono::NaiveDate) -> Result<()> {
    let mut tasks = store.load()?;
    state::auto_warm(&mut tasks, today);

    let task = tasks
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("Task {id} not found"))?;

    state::burn(task)?;
    println!(
        "Burned task {} [{}]: {}",
        task.id, task.state, task.title
    );

    store.save(&tasks)?;
    Ok(())
}

/// Evaporated → Melted
fn cmd_cool(store: &TaskStore, id: u32, today: chrono::NaiveDate) -> Result<()> {
    let mut tasks = store.load()?;
    state::auto_warm(&mut tasks, today);

    let task = tasks
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("Task {id} not found"))?;

    state::cool(task)?;
    println!(
        "Cooled task {} [{}]: {}",
        task.id, task.state, task.title
    );

    store.save(&tasks)?;
    Ok(())
}

/// 任意 → Iced
fn cmd_freeze(
    store: &TaskStore,
    id: u32,
    thaw_date_spec: Option<&str>,
    today: chrono::NaiveDate,
    config: &Config,
) -> Result<()> {
    let mut tasks = store.load()?;
    state::auto_warm(&mut tasks, today);

    let task = tasks
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("Task {id} not found"))?;

    let thaw_date = match thaw_date_spec {
        Some(spec) => parse_date_spec(spec, today)?,
        None => {
            // config からデフォルト解凍日数を取得
            today
                .checked_add_days(chrono::Days::new(config.defaults.thaw_days as u64))
                .ok_or_else(|| anyhow::anyhow!("Date overflow"))?
        }
    };

    state::freeze(task, thaw_date)?;
    println!(
        "Froze task {} [{}] until {}: {}",
        task.id, task.state, thaw_date, task.title
    );

    store.save(&tasks)?;
    Ok(())
}
