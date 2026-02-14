use clap::{Parser, Subcommand};

/// Kelvin - 熱力学メタファーのCLIタスク管理ツール
#[derive(Parser, Debug)]
#[command(name = "kelvin", version, about = "A thermodynamic task manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// タスクを追加する
    Add {
        /// タスクのタイトル
        title: String,
        /// タスクの説明
        #[arg(long = "desc")]
        description: Option<String>,
        /// 解凍予定日 (例: 3d, 1w, 2026-03-01)。指定するとIced状態で作成される
        #[arg(short = 'd', long = "date")]
        thaw_date: Option<String>,
        /// 期日 (例: 3d, 1w, 2026-03-01)
        #[arg(long = "due")]
        due_date: Option<String>,
    },

    /// タスクを編集する
    Edit {
        /// タスクID
        id: u32,
        /// 新しいタイトル
        #[arg(short = 't', long = "title")]
        title: Option<String>,
        /// 新しい説明
        #[arg(long = "desc")]
        description: Option<String>,
        /// 解凍予定日を変更 (例: 3d, 1w, 2026-03-01)
        #[arg(short = 'd', long = "date")]
        thaw_date: Option<String>,
        /// 期日を変更 (例: 3d, 1w, 2026-03-01)
        #[arg(long = "due")]
        due_date: Option<String>,
    },

    /// タスクの詳細を表示する
    Show {
        /// タスクID
        id: u32,
    },

    /// タスク一覧を表示する
    List {
        /// 凍結中(Iced)のタスクを表示する
        #[arg(long)]
        iced: bool,
        /// 全タスクを表示する
        #[arg(long)]
        all: bool,
    },

    /// タスクを着手可能状態にする (Melting/Iced → Melted)
    Warm {
        /// タスクID
        id: u32,
    },

    /// タスクを完了(気化)させる (Melted/Iced → Evaporated)
    Burn {
        /// タスクID
        id: u32,
    },

    /// 完了を取り消す (Evaporated → Melted)
    Cool {
        /// タスクID
        id: u32,
    },

    /// タスクを再冷凍する (→ Iced)
    Freeze {
        /// タスクID
        id: u32,
        /// 解凍予定日 (例: 3d, 1w, 2026-03-01)
        #[arg(short = 'd', long = "date")]
        thaw_date: Option<String>,
    },
}
