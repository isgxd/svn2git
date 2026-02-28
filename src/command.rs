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

#[cfg(test)]
mod tests {
    use clap::Parser;
    use std::path::PathBuf;

    use super::{Cli, Commands, HistoryCommands};

    #[test]
    fn test_parse_sync_command_with_paths() {
        let cli = Cli::parse_from([
            "svn2git",
            "sync",
            "--svn-dir",
            "d:/svn",
            "--git-dir",
            "d:/git",
        ]);

        match cli.command {
            Commands::Sync { svn_dir, git_dir } => {
                assert_eq!(svn_dir, Some(PathBuf::from("d:/svn")));
                assert_eq!(git_dir, Some(PathBuf::from("d:/git")));
            }
            _ => panic!("应解析为 Sync 命令"),
        }
    }

    #[test]
    fn test_parse_history_list_command() {
        let cli = Cli::parse_from(["svn2git", "history", "list"]);
        match cli.command {
            Commands::History { command } => match command {
                HistoryCommands::List => {}
                _ => panic!("应解析为 History List"),
            },
            _ => panic!("应解析为 History 命令"),
        }
    }

    #[test]
    fn test_parse_history_delete_command() {
        let cli = Cli::parse_from(["svn2git", "history", "delete", "3"]);
        match cli.command {
            Commands::History { command } => match command {
                HistoryCommands::Delete { id } => assert_eq!(id, 3),
                _ => panic!("应解析为 History Delete"),
            },
            _ => panic!("应解析为 History 命令"),
        }
    }
}
