//! Mock Git操作实现
//!
//! 提供Git操作的内存模拟实现，用于测试和开发环境

use crate::error::{Result, SyncError};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

/// Git文件状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum GitFileStatus {
    /// 未跟踪的新文件
    Untracked,
    /// 已暂存
    Staged,
    /// 已提交
    Committed,
    /// 已修改但未暂存
    Modified,
}

/// Mock Git仓库
///
/// 在内存中模拟Git仓库的状态和操作
#[derive(Debug, Clone)]
pub struct MockGitRepo {
    /// 仓库路径
    pub path: PathBuf,
    /// 文件状态映射：文件路径 -> 状态
    files: HashMap<String, GitFileStatus>,
    /// 提交历史
    commits: Vec<GitCommit>,
    /// 是否已初始化
    initialized: bool,
    /// 当前分支
    branch: String,
}

/// Git提交记录
#[derive(Debug, Clone, PartialEq)]
pub struct GitCommit {
    /// 提交哈希（简化版）
    pub hash: String,
    /// 提交消息
    pub message: String,
    /// 提交时间戳
    pub timestamp: String,
    /// 包含的文件列表
    pub files: Vec<String>,
}

impl MockGitRepo {
    /// 创建新的Mock Git仓库
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径
    ///
    /// # 返回值
    ///
    /// 返回新的MockGitRepo实例
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            files: HashMap::new(),
            commits: Vec::new(),
            initialized: false,
            branch: "main".to_string(),
        }
    }

    /// 初始化Git仓库
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 初始化成功
    /// * `Err(SyncError)` - 初始化失败
    pub fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Err(SyncError::App("Git仓库已经初始化".to_string()));
        }
        self.initialized = true;
        Ok(())
    }

    /// 检查仓库是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 添加文件到仓库（模拟文件创建）
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径（相对于仓库根目录）
    pub fn add_file(&mut self, file_path: &str) {
        self.files
            .insert(file_path.to_string(), GitFileStatus::Untracked);
    }

    /// 获取文件状态
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    ///
    /// # 返回值
    ///
    /// 返回文件状态，如果文件不存在则返回None
    pub fn get_file_status(&self, file_path: &str) -> Option<GitFileStatus> {
        self.files.get(file_path).cloned()
    }

    /// 模拟 `git add .` 操作
    ///
    /// 将所有未跟踪和已修改的文件添加到暂存区
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 添加成功
    /// * `Err(SyncError)` - 添加失败（如仓库未初始化）
    pub fn add_all(&mut self) -> Result<()> {
        if !self.initialized {
            return Err(SyncError::App("Git仓库未初始化".to_string()));
        }

        for (_, status) in self.files.iter_mut() {
            match status {
                GitFileStatus::Untracked | GitFileStatus::Modified => {
                    *status = GitFileStatus::Staged;
                }
                _ => {} // 已经是 Staged 或 Committed 状态的文件不需要更改
            }
        }
        Ok(())
    }

    /// 模拟 `git commit -m "message"` 操作
    ///
    /// 提交所有暂存的文件
    ///
    /// # 参数
    ///
    /// * `message` - 提交消息
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 提交成功
    /// * `Err(SyncError)` - 提交失败（如仓库未初始化、没有暂存文件等）
    pub fn commit(&mut self, message: &str) -> Result<()> {
        if !self.initialized {
            return Err(SyncError::App("Git仓库未初始化".to_string()));
        }

        // 收集所有状态为 Staged 的文件
        let staged_files: Vec<String> = self
            .files
            .iter()
            .filter(|(_, status)| matches!(status, GitFileStatus::Staged))
            .map(|(path, _)| path.clone())
            .collect();

        if staged_files.is_empty() {
            return Err(SyncError::App("没有暂存的文件可以提交".to_string()));
        }

        // 创建新的提交记录
        let commit = GitCommit {
            hash: format!("commit{}", self.commits.len() + 1),
            message: message.to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            files: staged_files.clone(),
        };

        // 添加到提交历史
        self.commits.push(commit);

        // 将这些文件的状态改为 Committed
        for file_path in staged_files {
            if let Some(status) = self.files.get_mut(&file_path) {
                *status = GitFileStatus::Committed;
            }
        }

        Ok(())
    }

    /// 获取提交历史
    pub fn get_commits(&self) -> &Vec<GitCommit> {
        &self.commits
    }

    /// 获取当前分支名
    pub fn get_branch(&self) -> &str {
        &self.branch
    }

    /// 检查工作目录是否干净（没有未提交的更改）
    pub fn is_working_directory_clean(&self) -> bool {
        self.files
            .values()
            .all(|status| matches!(status, GitFileStatus::Committed))
    }

    /// 模拟文件修改
    ///
    /// 将已提交的文件标记为已修改
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 修改成功
    /// * `Err(SyncError)` - 文件不存在或未提交
    pub fn modify_file(&mut self, file_path: &str) -> Result<()> {
        match self.files.get_mut(file_path) {
            Some(GitFileStatus::Committed) => {
                *self.files.get_mut(file_path).unwrap() = GitFileStatus::Modified;
                Ok(())
            }
            Some(_) => Err(SyncError::App(format!(
                "文件 {} 不是已提交状态，无法修改",
                file_path
            ))),
            None => Err(SyncError::App(format!("文件 {} 不存在", file_path))),
        }
    }
}

/// Mock Git操作实现
///
/// 使用内存状态模拟Git操作，用于测试
#[derive(Debug, Clone)]
pub struct MockGitOperations {
    /// 存储所有Mock仓库
    repos: Arc<RwLock<HashMap<String, MockGitRepo>>>,
}

impl MockGitOperations {
    /// 创建新的Mock Git操作实例
    ///
    /// # 返回值
    ///
    /// 返回新的MockGitOperations实例
    pub fn new() -> Self {
        Self {
            repos: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 手动添加文件到Mock仓库状态中
    ///
    /// 这个方法用于测试，当在文件系统中创建了文件后，
    /// 需要手动通知Mock系统该文件的存在
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库路径
    /// * `file_path` - 相对于仓库根目录的文件路径
    ///
    /// # 示例
    ///
    /// ```
    /// use svn2git::{MockGitOperations, GitOperations};
    /// use std::path::PathBuf;
    ///
    /// let git_ops = MockGitOperations::new();
    /// let repo_path = PathBuf::from("/test/repo");
    /// git_ops.init(&repo_path).expect("初始化失败");
    ///
    /// // 创建真实文件后，通知Mock系统
    /// git_ops.add_file_to_mock(&repo_path, "test.txt");
    /// git_ops.add_all(&repo_path).expect("添加失败");
    /// git_ops.commit(&repo_path, "测试提交").expect("提交失败");
    /// ```
    pub fn add_file_to_mock(&self, repo_path: &Path, file_path: &str) -> Result<()> {
        let mut repo = self.get_or_create_repo(repo_path);
        repo.add_file(file_path);
        self.update_repo(repo_path, repo)?;
        Ok(())
    }

    /// 获取或创建Mock仓库
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径
    ///
    /// # 返回值
    ///
    /// 返回Mock仓库的副本
    fn get_or_create_repo(&self, path: &Path) -> MockGitRepo {
        let path_str = path.to_string_lossy().to_string();

        // 首先尝试读取锁
        {
            let repos = self.repos.read().unwrap();
            if let Some(repo) = repos.get(&path_str) {
                return repo.clone();
            }
        }

        // 如果不存在，则创建新的
        {
            let mut repos = self.repos.write().unwrap();
            repos
                .entry(path_str)
                .or_insert_with(|| MockGitRepo::new(path.to_path_buf()))
                .clone()
        }
    }

    /// 更新Mock仓库
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径
    /// * `repo` - 更新后的仓库
    fn update_repo(&self, path: &Path, repo: MockGitRepo) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let mut repos = self.repos.write().unwrap();
        repos.insert(path_str, repo);
        Ok(())
    }

    /// 获取Mock仓库状态（用于测试验证）
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径
    ///
    /// # 返回值
    ///
    /// 返回仓库状态的克隆
    pub fn get_repo_state(&self, path: &Path) -> Option<MockGitRepo> {
        let path_str = path.to_string_lossy().to_string();
        let repos = self.repos.read().unwrap();
        repos.get(&path_str).cloned()
    }
}

impl Default for MockGitOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl super::git_operations::GitOperations for MockGitOperations {
    fn init(&self, path: &Path) -> Result<()> {
        let mut repo = self.get_or_create_repo(path);
        let result = repo.init();
        if result.is_ok() {
            self.update_repo(path, repo)?;
        }
        result
    }

    fn config_user(&self, _path: &Path, _name: &str, _email: &str) -> Result<()> {
        // Mock实现不需要真实的用户配置
        Ok(())
    }

    fn add_all(&self, path: &Path) -> Result<()> {
        let mut repo = self.get_or_create_repo(path);
        let result = repo.add_all();
        self.update_repo(path, repo)?;
        result
    }

    fn commit(&self, path: &Path, message: &str) -> Result<()> {
        let mut repo = self.get_or_create_repo(path);
        let result = repo.commit(message);
        self.update_repo(path, repo)?;
        result
    }

    fn status(&self, path: &Path) -> Result<String> {
        let repo = self.get_or_create_repo(path);
        if repo.is_working_directory_clean() {
            Ok(String::new())
        } else {
            // 模拟Git状态输出
            Ok("?? some_untracked_file.txt\nM some_modified_file.txt".to_string())
        }
    }

    fn log(&self, path: &Path, count: Option<usize>) -> Result<String> {
        let repo = self.get_or_create_repo(path);
        let commits = repo.get_commits();

        let limit = count.unwrap_or(commits.len());
        let limited_commits: Vec<_> = commits.iter().rev().take(limit).collect();

        let mut result = String::new();
        for commit in limited_commits {
            result.push_str(&format!("{} {}\n", commit.hash, commit.message));
        }

        Ok(result)
    }

    fn is_clean(&self, path: &Path) -> Result<bool> {
        let repo = self.get_or_create_repo(path);
        Ok(repo.is_working_directory_clean())
    }
}

#[cfg(test)]
mod tests {
    use super::super::git_operations::GitOperations;
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mock_git_repo_creation() {
        let repo = MockGitRepo::new(PathBuf::from("/test"));
        assert!(!repo.is_initialized());
        assert_eq!(repo.get_branch(), "main");
        assert!(repo.get_commits().is_empty());
    }

    #[test]
    fn test_mock_git_repo_init() {
        let mut repo = MockGitRepo::new(PathBuf::from("/test"));

        assert!(repo.init().is_ok());
        assert!(repo.is_initialized());

        assert!(repo.init().is_err());
    }

    #[test]
    fn test_add_file_and_status() {
        let mut repo = MockGitRepo::new(PathBuf::from("/test"));
        repo.add_file("test.txt");

        assert_eq!(
            repo.get_file_status("test.txt"),
            Some(GitFileStatus::Untracked)
        );
        assert_eq!(repo.get_file_status("nonexistent.txt"), None);
    }

    #[test]
    fn test_add_all_untracked_files() {
        let mut repo = MockGitRepo::new(PathBuf::from("/test"));
        repo.init().expect("初始化失败");

        repo.add_file("file1.txt");
        repo.add_file("file2.txt");

        assert!(repo.add_all().is_ok());
        assert_eq!(
            repo.get_file_status("file1.txt"),
            Some(GitFileStatus::Staged)
        );
        assert_eq!(
            repo.get_file_status("file2.txt"),
            Some(GitFileStatus::Staged)
        );
    }

    #[test]
    fn test_commit_staged_files() {
        let mut repo = MockGitRepo::new(PathBuf::from("/test"));
        repo.init().expect("初始化失败");

        repo.add_file("test.txt");
        repo.add_all().expect("添加失败");

        assert!(repo.commit("测试提交").is_ok());
        assert_eq!(repo.get_commits().len(), 1);
        assert_eq!(
            repo.get_file_status("test.txt"),
            Some(GitFileStatus::Committed)
        );
        assert!(repo.is_working_directory_clean());
    }

    #[test]
    fn test_modify_committed_file() {
        let mut repo = MockGitRepo::new(PathBuf::from("/test"));
        repo.init().expect("初始化失败");

        repo.add_file("test.txt");
        repo.add_all().expect("添加失败");
        repo.commit("初始提交").expect("提交失败");

        assert!(repo.modify_file("test.txt").is_ok());
        assert_eq!(
            repo.get_file_status("test.txt"),
            Some(GitFileStatus::Modified)
        );
        assert!(!repo.is_working_directory_clean());
    }

    #[test]
    fn test_mock_git_operations() {
        let ops = MockGitOperations::new();
        let path = PathBuf::from("/test/repo");

        // 测试初始化
        assert!(ops.init(&path).is_ok());

        // 测试用户配置
        assert!(
            ops.config_user(&path, "测试用户", "test@example.com")
                .is_ok()
        );

        // 测试状态查询
        assert!(ops.status(&path).is_ok());

        // 测试日志查询
        assert!(ops.log(&path, None).is_ok());

        // 测试工作目录状态
        assert!(ops.is_clean(&path).is_ok());
    }

    #[test]
    fn test_add_file_to_mock() {
        let ops = MockGitOperations::new();
        let path = PathBuf::from("/test/repo");

        ops.init(&path).expect("初始化失败");
        ops.add_file_to_mock(&path, "test.txt")
            .expect("添加文件失败");

        let repo_state = ops.get_repo_state(&path).unwrap();
        assert_eq!(
            repo_state.get_file_status("test.txt"),
            Some(GitFileStatus::Untracked)
        );
    }
}
