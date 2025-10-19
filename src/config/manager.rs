use std::path::PathBuf;

use crate::{
    config::reocrd::{self, HistoryRecord},
    error::{Result, SyncError},
};

/// 配置文件
pub struct HistoryManager<S: FileStorage> {
    records: Vec<HistoryRecord>,
    storage: S,
}

/// 文件存储
#[cfg_attr(test, mockall::automock)]
pub trait FileStorage {
    /// 加载文件
    fn load(&self) -> Result<Vec<HistoryRecord>>;
    /// 保存文件
    fn save(&self, records: &[HistoryRecord]) -> Result<()>;
}

impl<S: FileStorage> HistoryManager<S> {
    /// 创建一个新的配置
    pub fn new(storage: S) -> Result<Self> {
        Ok(Self {
            records: storage.load()?,
            storage,
        })
    }

    /// 记录是否为空
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 获取记录列表
    pub fn records(&self) -> &[HistoryRecord] {
        &self.records
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        self.storage.save(&self.records)
    }

    /// 添加记录
    ///
    /// # 参数
    ///
    /// * `svn_path`: SVN 路径
    /// * `git_path`: Git 路径
    pub fn add_record(&mut self, svn_path: PathBuf, git_path: PathBuf) {
        // 删除重复记录
        self.records.retain(|r| !r.path_eq(&svn_path, &git_path));

        let new_record = HistoryRecord::new(self.records.len() + 1, svn_path, git_path);
        self.records.push(new_record);
        self.records.sort_by(reocrd::cmp_last_used);
    }

    /// 删除记录
    ///
    /// # 参数
    ///
    /// * `index`: 删除的路径的索引
    ///
    /// # 返回
    ///
    /// 如果删除成功，返回 `Ok(())`，否则返回 `Err(SyncError::ConfigError(String))`
    pub fn remove_record(&mut self, index: usize) -> Result<()> {
        if index >= self.records.len() {
            return Err(SyncError::App("索引超出范围".into()));
        }
        self.records.remove(index);
        println!("已删除记录 {index}");
        self.save()
    }

    /// 列出所有记录
    pub fn list(&self) {
        if self.records.is_empty() {
            println!("还没有记录");
            return;
        }

        reocrd::print_title();
        for record in &self.records {
            println!("{record}");
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use crate::config::{HistoryManager, MockFileStorage};
    #[cfg(test)]
    use std::{fs, path::PathBuf};

    #[test]
    fn test_add_and_list_pairs() {
        let dir = tempfile::tempdir().unwrap();
        let svn_path = dir.path().join("svn");
        let git_path = dir.path().join("git");

        fs::create_dir(&svn_path).unwrap();
        fs::create_dir(&git_path).unwrap();

        let mut disk = MockFileStorage::new();
        disk.expect_load().returning(|| Ok(vec![]));
        let mut config = HistoryManager::new(disk).unwrap();
        config.add_record(svn_path.clone(), git_path.clone());

        assert_eq!(config.records.len(), 1);
        assert!(config.records[0].path_eq(&svn_path, &git_path));
    }

    #[test]
    fn test_remove_pair() {
        let mut disk = MockFileStorage::new();
        disk.expect_load().returning(|| Ok(vec![]));
        disk.expect_save().returning(|_| Ok(()));
        let mut config = HistoryManager::new(disk).unwrap();
        config.add_record(PathBuf::from("svn1"), PathBuf::from("git1"));
        config.add_record(PathBuf::from("svn2"), PathBuf::from("git2"));

        assert!(config.remove_record(0).is_ok());
        assert_eq!(config.records.len(), 1);
        assert!(config.records[0].path_eq(&PathBuf::from("svn2"), &PathBuf::from("git2")));
    }

    #[test]
    fn test_list_history() {
        let mut disk = MockFileStorage::new();
        disk.expect_load().returning(|| Ok(vec![]));

        let mut config = HistoryManager::new(disk).unwrap();
        config.add_record(PathBuf::from("D:\\svn1"), PathBuf::from("D:\\git1"));
        config.add_record(PathBuf::from("D:\\svn2"), PathBuf::from("D:\\git2"));

        config.list();
    }
}
