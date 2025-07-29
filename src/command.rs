use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// 命令
#[derive(Debug, Parser)]
#[command(name = "svn2git", version, about = "同步 SVN 到 Git 仓库的工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 同步命令
    Sync {
        #[arg(short, long)]
        svn_dir: Option<PathBuf>,
        #[arg(short, long)]
        git_dir: Option<PathBuf>,
    },

    /// 历史记录命令
    History {
        #[command(subcommand)]
        command: HistoryCommands,
    },
}

/// 历史记录命令
#[derive(Debug, Subcommand)]
pub enum HistoryCommands {
    /// 列出历史记录
    List,
    /// 按 ID 删除历史记录
    Delete { id: usize },
}
