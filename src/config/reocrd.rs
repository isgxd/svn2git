use std::{fmt::Display, path::PathBuf};

use chrono::{DateTime, Local, Utc};

use serde::{Deserialize, Serialize};

/// 同步配置
pub struct SyncConfig {
    pub svn_dir: PathBuf,
    pub git_dir: PathBuf,
}

impl SyncConfig {
    /// 创建一个新的同步配置
    pub fn new(svn_dir: PathBuf, git_dir: PathBuf) -> Self {
        Self { svn_dir, git_dir }
    }
}

/// 历史记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryRecord {
    id: usize,
    svn_path: PathBuf,
    git_path: PathBuf,
    last_used: DateTime<Utc>,
}

impl HistoryRecord {
    /// 创建一个新的历史记录
    ///
    /// # 参数
    ///
    /// * `id`: 记录的编号
    /// * `svn_path`: SVN 路径
    /// * `git_path`: Git 路径
    pub fn new(id: usize, svn_path: PathBuf, git_path: PathBuf) -> Self {
        Self::new_with(id, svn_path, git_path, Utc::now())
    }

    /// 创建一个新的历史记录
    ///
    /// # 参数
    ///
    /// * `id`: 记录的编号
    /// * `svn_path`: SVN 路径
    /// * `git_path`: Git 路径
    /// * `last_used`: 最后使用时间
    pub fn new_with(
        id: usize,
        svn_path: PathBuf,
        git_path: PathBuf,
        last_used: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            svn_path,
            git_path,
            last_used,
        }
    }

    /// 检查 id 是否相同
    ///
    /// # 参数
    ///
    /// * `id`: 要比较的 id
    pub fn id_eq(&self, id: usize) -> bool {
        self.id == id
    }

    /// 检查是否包含相同的记录
    ///
    /// # 参数
    ///
    /// * `svn_path`: SVN 路径
    /// * `git_path`: Git 路径
    pub fn path_eq(&self, svn_path: &PathBuf, git_path: &PathBuf) -> bool {
        self.svn_path.eq(svn_path) && self.git_path.eq(git_path)
    }

    /// 转换为 `SyncConfig`
    pub fn to_sync_config(&self) -> SyncConfig {
        SyncConfig::new(self.svn_path.clone(), self.git_path.clone())
    }
}

/// 按照最后使用时间排序
pub fn cmp_last_used(a: &HistoryRecord, b: &HistoryRecord) -> std::cmp::Ordering {
    a.last_used.cmp(&b.last_used)
}

/// 打印标题行
pub fn print_title() {
    println!("ID \tSVN Path \tGit Path \tLast Used");
}

impl Display for HistoryRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} \t{} \t{} \t{}",
            self.id,
            self.svn_path.to_str().unwrap(),
            self.git_path.to_str().unwrap(),
            self.last_used
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
        )
    }
}
