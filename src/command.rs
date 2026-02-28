use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// 命令
#[derive(Debug, Parser)]
#[command(
    name = "svn2git",
    version,
    about = "同步 SVN 到 Git 仓库的工具",
    long_about = "将 SVN 提交按顺序同步为 Git 提交。支持交互式选择历史配置，也支持通过参数直传目录。",
    arg_required_else_help = true,
    after_help = "示例:\n  svn2git sync --svn-dir D:\\svn_wc --git-dir D:\\git_repo\n  svn2git sync\n  svn2git history list\n  svn2git history delete 0"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 同步命令
    #[command(
        about = "执行 SVN -> Git 同步",
        long_about = "读取 SVN 日志并逐条更新工作副本，然后在 Git 中生成对应提交。"
    )]
    Sync {
        #[arg(
            short,
            long,
            value_name = "PATH",
            help = "SVN 工作副本目录（不传则走历史选择或交互输入）",
            long_help = "SVN 工作副本目录。\n- 与 --git-dir 同时传入：直接使用这组配置同步\n- 不传：若有历史记录会先让你选择；无历史则交互输入"
        )]
        svn_dir: Option<PathBuf>,

        #[arg(short, long)]
        #[arg(
            short,
            long,
            value_name = "PATH",
            help = "Git 仓库目录（留空时默认与 SVN 目录相同）",
            long_help = "Git 仓库目录。\n- 与 --svn-dir 同时传入：直接使用这组配置同步\n- 交互输入时留空：默认使用 SVN 目录"
        )]
        git_dir: Option<PathBuf>,
    },

    /// 历史记录命令
    #[command(about = "查看或删除历史配置")]
    History {
        #[command(subcommand)]
        command: HistoryCommands,
    },
}

/// 历史记录命令
#[derive(Debug, Subcommand)]
pub enum HistoryCommands {
    /// 列出历史记录
    #[command(about = "列出历史同步配置")]
    List,

    /// 按 ID 删除历史记录
    #[command(about = "删除指定索引的历史记录（索引可通过 history list 查看）")]
    Delete { id: usize },
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::error::ErrorKind;
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

    #[test]
    fn test_help_contains_examples() {
        let err = Cli::try_parse_from(["svn2git", "--help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
        let msg = err.to_string();
        assert!(msg.contains("示例:"));
        assert!(msg.contains("svn2git sync"));
        assert!(msg.contains("history list"));
    }
}
