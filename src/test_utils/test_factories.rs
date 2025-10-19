//! 测试工厂模块
//!
//! 提供创建测试对象的标准方法，避免依赖配置文件

use std::path::{Path, PathBuf};

use crate::{
    config::SyncConfig,
    ops::{GitOperations, MockGitOperations, ProviderType},
};

/// 测试对象工厂
///
/// 提供创建各种测试对象的标准方法
pub struct TestFactory;

impl TestFactory {
    /// 创建测试用的SyncConfig
    ///
    /// # 参数
    ///
    /// * `use_real_git` - 是否使用真实Git实现
    ///
    /// # 返回值
    ///
    /// 返回测试用的SyncConfig
    pub fn create_sync_config(use_real_git: bool) -> SyncConfig {
        let svn_dir = TestFactory::test_path(&["svn"]);
        let git_dir = TestFactory::test_path(&["git"]);
        let git_provider = if use_real_git {
            ProviderType::Real
        } else {
            ProviderType::Mock
        };
        SyncConfig::with_git_provider(svn_dir, git_dir, git_provider)
    }

    /// 创建测试路径
    ///
    /// # 参数
    ///
    /// * `components` - 路径组件
    ///
    /// # 返回值
    ///
    /// 返回测试路径
    pub fn test_path(components: &[&str]) -> PathBuf {
        let mut path = PathBuf::from("/test");
        for component in components {
            path.push(component);
        }
        path
    }

    /// 创建标准测试数据集
    ///
    /// # 返回值
    ///
    /// 返回包含标准测试数据的结构体
    pub fn create_test_data() -> TestData {
        TestData {
            svn_dir: TestFactory::test_path(&["svn"]),
            git_dir: TestFactory::test_path(&["git"]),
            commits: vec![("1", "初始提交"), ("2", "添加功能"), ("3", "修复bug")],
        }
    }
}

/// 标准测试数据集
#[derive(Debug, Clone)]
pub struct TestData {
    /// SVN目录路径
    pub svn_dir: PathBuf,
    /// Git目录路径
    pub git_dir: PathBuf,
    /// 模拟的提交记录
    pub commits: Vec<(&'static str, &'static str)>,
}

impl TestData {
    /// 创建使用Mock Git的SyncConfig
    ///
    /// # 返回值
    ///
    /// 返回配置好的SyncConfig
    pub fn create_mock_sync_config(&self) -> SyncConfig {
        SyncConfig::with_git_provider(
            self.svn_dir.clone(),
            self.git_dir.clone(),
            ProviderType::Mock,
        )
    }

    /// 创建使用真实Git的SyncConfig
    ///
    /// # 返回值
    ///
    /// 返回配置好的SyncConfig
    pub fn create_real_sync_config(&self) -> SyncConfig {
        SyncConfig::with_git_provider(
            self.svn_dir.clone(),
            self.git_dir.clone(),
            ProviderType::Real,
        )
    }
}

/// Git操作测试辅助工具
pub struct GitTestHelper;

impl GitTestHelper {
    /// 创建Mock Git操作实例
    ///
    /// # 返回值
    ///
    /// 返回MockGitOperations实例
    pub fn create_mock_git() -> MockGitOperations {
        MockGitOperations::new()
    }

    /// 在Mock Git中添加测试文件
    ///
    /// # 参数
    ///
    /// * `git_ops` - Mock Git操作实例
    /// * `repo_path` - 仓库路径
    /// * `files` - 要添加的文件列表
    pub fn add_test_files(git_ops: &MockGitOperations, repo_path: &Path, files: &[&str]) {
        for file in files {
            git_ops
                .add_file_to_mock(repo_path, file)
                .expect("添加文件到Mock失败");
        }
    }

    /// 模拟完整的Git工作流程
    ///
    /// # 参数
    ///
    /// * `git_ops` - Mock Git操作实例
    /// * `repo_path` - 仓库路径
    /// * `files` - 要提交的文件列表
    /// * `message` - 提交消息
    pub fn commit_files(
        git_ops: &MockGitOperations,
        repo_path: &Path,
        files: &[&str],
        message: &str,
    ) {
        // 初始化仓库
        git_ops.init(repo_path).expect("初始化Mock仓库失败");

        // 添加文件
        GitTestHelper::add_test_files(git_ops, repo_path, files);

        // 添加到暂存区
        git_ops.add_all(repo_path).expect("添加到暂存区失败");

        // 提交
        git_ops.commit(repo_path, message).expect("提交失败");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sync_config() {
        let config = TestFactory::create_sync_config(false);
        assert_eq!(config.git_provider, ProviderType::Mock);

        let config = TestFactory::create_sync_config(true);
        assert_eq!(config.git_provider, ProviderType::Real);
    }

    #[test]
    fn test_test_data() {
        let data = TestFactory::create_test_data();
        assert_eq!(data.commits.len(), 3);
        assert_eq!(data.commits[0], ("1", "初始提交"));

        let config = data.create_mock_sync_config();
        assert_eq!(config.git_provider, ProviderType::Mock);
    }

    #[test]
    fn test_git_test_helper() {
        let git_ops = GitTestHelper::create_mock_git();
        let repo_path = TestFactory::test_path(&["repo"]);

        GitTestHelper::commit_files(
            &git_ops,
            &repo_path,
            &["test.txt", "src/main.rs"],
            "测试提交",
        );

        // 验证提交成功
        let log = git_ops.log(&repo_path, None).expect("获取日志失败");
        assert!(log.contains("测试提交"));
    }
}
