//! 测试工厂模块
//!
//! 提供创建测试场景的工厂函数，简化测试代码编写

use std::path::PathBuf;
#[cfg(test)]
use svn2git::{GitOperations, MockGitOperations};
use tempfile::TempDir;

/// 测试场景工厂
pub struct TestFactory;

impl TestFactory {
    /// 创建标准的Git仓库测试场景
    ///
    /// # 返回值
    ///
    /// 返回 (临时目录, Git目录, MockGit操作实例)
    ///
    /// # 示例
    ///
    /// ```
    /// use tests::test_factories::TestFactory;
    ///
    /// let (_temp_dir, git_dir, git_ops) = TestFactory::create_git_repo();
    /// // 现在可以开始测试了
    /// ```
    pub fn create_git_repo() -> (TempDir, PathBuf, MockGitOperations) {
        let temp_dir = TempDir::new().expect("创建临时目录失败");
        let git_dir = temp_dir.path().join("git");
        std::fs::create_dir(&git_dir).expect("创建Git目录失败");

        let git_ops = MockGitOperations::new();
        git_ops.init(&git_dir).expect("初始化Git仓库失败");
        git_ops
            .config_user(&git_dir, "测试用户", "test@example.com")
            .expect("配置Git用户失败");

        (temp_dir, git_dir, git_ops)
    }

    /// 创建包含文件的Git仓库测试场景
    ///
    /// # 参数
    ///
    /// * `files` - 文件名和内容的向量
    ///
    /// # 返回值
    ///
    /// 返回 (临时目录, Git目录, MockGit操作实例, 文件路径列表)
    ///
    /// # 示例
    ///
    /// ```
    /// use tests::test_factories::TestFactory;
    ///
    /// let files = vec![("test.txt", "测试内容"), ("src/main.rs", "fn main() {}")];
    /// let (_temp_dir, git_dir, git_ops, file_paths) = TestFactory::create_git_repo_with_files(files);
    /// ```
    pub fn create_git_repo_with_files(
        files: Vec<(&str, &str)>,
    ) -> (TempDir, PathBuf, MockGitOperations, Vec<PathBuf>) {
        let (temp_dir, git_dir, git_ops) = Self::create_git_repo();
        let mut file_paths = Vec::new();

        for (filename, _content) in files {
            let file_path = git_dir.join(filename);

            // 通知Mock系统文件存在（不创建真实文件）
            let _ = git_ops.add_file_to_mock(&git_dir, filename);
            file_paths.push(file_path);
        }

        (temp_dir, git_dir, git_ops, file_paths)
    }

    /// 创建包含提交的Git仓库测试场景
    ///
    /// # 参数
    ///
    /// * `commits` - 提交消息和文件变化的向量
    ///
    /// # 返回值
    ///
    /// 返回 (临时目录, Git目录, MockGit操作实例)
    ///
    /// # 示例
    ///
    /// ```
    /// use tests::test_factories::TestFactory;
    ///
    /// let commits = vec![
    ///     ("初始提交", vec![("README.md", "# 项目")]),
    ///     ("添加代码", vec![("src/main.rs", "fn main() {}")]),
    /// ];
    /// let (_temp_dir, git_dir, git_ops) = TestFactory::create_git_repo_with_commits(commits);
    /// ```
    pub fn create_git_repo_with_commits(
        commits: Vec<(&str, Vec<(&str, &str)>)>,
    ) -> (TempDir, PathBuf, MockGitOperations) {
        let (temp_dir, git_dir, git_ops) = Self::create_git_repo();

        for (commit_message, files) in commits {
            // 通知Mock系统文件存在（不创建真实文件）
            for (filename, _content) in files {
                let _ = git_ops.add_file_to_mock(&git_dir, filename);
            }

            // 提交更改
            git_ops.add_all(&git_dir).expect("添加文件到暂存区失败");
            git_ops
                .commit(&git_dir, commit_message)
                .expect(&format!("提交失败: {}", commit_message));
        }

        (temp_dir, git_dir, git_ops)
    }

    /// 创建SVN同步测试场景
    ///
    /// # 参数
    ///
    /// * `svn_logs` - SVN版本号和消息的向量
    ///
    /// # 返回值
    ///
    /// 返回 (临时目录, Git目录, SVN目录, MockGit操作实例)
    ///
    /// # 示例
    ///
    /// ```
    /// use tests::test_factories::TestFactory;
    ///
    /// let svn_logs = vec![("123", "修复bug"), ("124", "添加功能")];
    /// let (_temp_dir, git_dir, svn_dir, git_ops) = TestFactory::create_svn_sync_scenario(svn_logs);
    /// ```
    pub fn create_svn_sync_scenario(
        svn_logs: Vec<(&str, &str)>,
    ) -> (TempDir, PathBuf, PathBuf, MockGitOperations) {
        let temp_dir = TempDir::new().expect("创建临时目录失败");
        let git_dir = temp_dir.path().join("git");
        let svn_dir = temp_dir.path().join("svn");

        std::fs::create_dir(&git_dir).expect("创建Git目录失败");
        std::fs::create_dir(&svn_dir).expect("创建SVN目录失败");

        let git_ops = MockGitOperations::new();
        git_ops.init(&git_dir).expect("初始化Git仓库失败");
        git_ops
            .config_user(&git_dir, "SVN同步用户", "sync@example.com")
            .expect("配置Git用户失败");

        for (version, message) in svn_logs {
            // 模拟SVN更新：创建SVN版本文件
            let svn_file = svn_dir.join(format!("svn_r{}.txt", version));
            std::fs::write(&svn_file, format!("SVN版本{}的内容", version))
                .expect("创建SVN文件失败");

            // 模拟同步到Git：通知Mock系统Git文件存在（不创建真实文件）
            let git_filename = format!("sync_r{}.txt", version);
            let _ = git_ops.add_file_to_mock(&git_dir, &git_filename);

            // 提交到Git
            let commit_message = format!("SVN: {}", message);
            git_ops.add_all(&git_dir).expect("添加同步文件失败");
            git_ops
                .commit(&git_dir, &commit_message)
                .expect(&format!("SVN同步提交失败: r{}", version));
        }

        (temp_dir, git_dir, svn_dir, git_ops)
    }

    /// 验证Git仓库状态的工具函数
    ///
    /// # 参数
    ///
    /// * `git_ops` - Git操作实例
    /// * `git_dir` - Git目录路径
    /// * `expected_commits` - 期望的提交消息列表
    /// * `should_be_clean` - 期望工作目录是否干净
    ///
    /// # 示例
    ///
    /// ```
    /// use tests::test_factories::TestFactory;
    ///
    /// let (_temp_dir, git_dir, git_ops) = TestFactory::create_git_repo();
    /// TestFactory::assert_git_state(&git_ops, &git_dir, vec![], true);
    /// ```
    pub fn assert_git_state(
        git_ops: &MockGitOperations,
        git_dir: &PathBuf,
        expected_commits: Vec<&str>,
        should_be_clean: bool,
    ) {
        // 检查工作目录状态
        let is_clean = git_ops.is_clean(git_dir).expect("检查工作目录状态失败");
        assert_eq!(
            is_clean, should_be_clean,
            "工作目录干净状态不匹配，期望: {}, 实际: {}",
            should_be_clean, is_clean
        );

        // 检查提交历史
        let logs = git_ops.log(git_dir, None).expect("获取提交历史失败");
        let log_lines: Vec<&str> = logs.lines().collect();

        assert_eq!(
            log_lines.len(),
            expected_commits.len(),
            "提交数量不匹配，期望: {}, 实际: {}",
            expected_commits.len(),
            log_lines.len()
        );

        // 验证每个提交消息
        for (expected_message, actual_log) in expected_commits.iter().zip(log_lines.iter()) {
            assert!(
                actual_log.contains(expected_message),
                "提交历史中缺少期望的消息: {}",
                expected_message
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：工厂函数应该能创建基本的Git仓库场景
    #[test]
    fn test_factory_create_git_repo() {
        let (_temp_dir, git_dir, git_ops) = TestFactory::create_git_repo();

        // 验证仓库已初始化
        let status = git_ops.status(&git_dir).expect("获取状态失败");
        assert!(status.trim().is_empty());

        // 验证工作目录干净
        let is_clean = git_ops.is_clean(&git_dir).expect("检查状态失败");
        assert!(is_clean);
    }

    /// 测试：工厂函数应该能创建包含文件的Git仓库场景
    #[test]
    fn test_factory_create_git_repo_with_files() {
        let files = vec![("test.txt", "测试内容"), ("src/main.rs", "fn main() {}")];
        let files_count = files.len();
        let (_temp_dir, git_dir, git_ops, file_paths) =
            TestFactory::create_git_repo_with_files(files);

        // 验证文件路径已生成（不验证真实文件存在，因为使用Mock）
        assert_eq!(file_paths.len(), files_count, "文件路径数量应该匹配");

        // 验证具体的文件路径
        let expected_filenames = ["test.txt", "src/main.rs"];
        for (i, expected_filename) in expected_filenames.iter().enumerate() {
            let expected_path = git_dir.join(expected_filename);
            assert_eq!(file_paths[i], expected_path, "文件路径应该正确");
        }

        // 验证Git状态显示有未跟踪文件
        let status = git_ops.status(&git_dir).expect("获取状态失败");
        assert!(!status.trim().is_empty(), "应该有未跟踪的文件");

        // 验证工作目录不干净
        let is_clean = git_ops.is_clean(&git_dir).expect("检查状态失败");
        assert!(!is_clean, "有未跟踪文件时工作目录不应该干净");
    }

    /// 测试：工厂函数应该能创建包含提交的Git仓库场景
    #[test]
    fn test_factory_create_git_repo_with_commits() {
        let commits = vec![
            ("初始提交", vec![("README.md", "# 项目")]),
            ("添加代码", vec![("src/main.rs", "fn main() {}")]),
        ];
        let (_temp_dir, git_dir, git_ops) =
            TestFactory::create_git_repo_with_commits(commits.clone());

        // 验证提交历史（注意：Git log 默认显示最新提交在前，所以需要反转期望顺序）
        let expected_commits: Vec<&str> = commits.iter().map(|(msg, _)| *msg).rev().collect();
        TestFactory::assert_git_state(&git_ops, &git_dir, expected_commits, true);
    }

    /// 测试：工厂函数应该能创建SVN同步场景
    #[test]
    fn test_factory_create_svn_sync_scenario() {
        let svn_logs = vec![("123", "修复bug"), ("124", "添加功能")];
        let (_temp_dir, git_dir, svn_dir, git_ops) =
            TestFactory::create_svn_sync_scenario(svn_logs);

        // 验证SVN文件存在（SVN文件仍然创建真实文件用于模拟SVN仓库）
        assert!(svn_dir.join("svn_r123.txt").exists());
        assert!(svn_dir.join("svn_r124.txt").exists());

        // 验证提交历史（Git文件使用Mock，不验证真实文件存在）
        // 注意：Git log 默认显示最新提交在前
        let expected_commits = vec!["SVN: 添加功能", "SVN: 修复bug"];
        TestFactory::assert_git_state(&git_ops, &git_dir, expected_commits, true);
    }
}
