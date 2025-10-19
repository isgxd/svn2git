//! Git操作抽象接口
//!
//! 定义Git操作的统一接口，支持真实Git命令和Mock实现

use crate::error::Result;
use std::path::Path;

/// Git操作抽象特征
///
/// 提供所有Git相关操作的统一接口，支持真实实现和Mock实现
pub trait GitOperations {
    /// 初始化Git仓库
    ///
    /// # 参数
    ///
    /// * `path` - Git仓库路径
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 初始化成功
    /// * `Err(SyncError)` - 初始化失败
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use svn2git::{GitOperations, GitProvider};
    /// use std::path::PathBuf;
    ///
    /// let git_ops = GitProvider::auto();
    /// git_ops.init(&PathBuf::from("/test/repo")).expect("初始化失败");
    /// ```
    fn init(&self, path: &Path) -> Result<()>;

    /// 配置Git用户信息
    ///
    /// # 参数
    ///
    /// * `path` - Git仓库路径
    /// * `name` - 用户名
    /// * `email` - 用户邮箱
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 配置成功
    /// * `Err(SyncError)` - 配置失败
    fn config_user(&self, path: &Path, name: &str, email: &str) -> Result<()>;

    /// 添加所有更改到暂存区
    ///
    /// # 参数
    ///
    /// * `path` - Git仓库路径
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 添加成功
    /// * `Err(SyncError)` - 添加失败
    fn add_all(&self, path: &Path) -> Result<()>;

    /// 提交更改
    ///
    /// # 参数
    ///
    /// * `path` - Git仓库路径
    /// * `message` - 提交消息
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 提交成功
    /// * `Err(SyncError)` - 提交失败
    fn commit(&self, path: &Path, message: &str) -> Result<()>;

    /// 获取Git状态
    ///
    /// # 参数
    ///
    /// * `path` - Git仓库路径
    ///
    /// # 返回值
    ///
    /// * `Ok(String)` - Git状态输出
    /// * `Err(SyncError)` - 获取状态失败
    fn status(&self, path: &Path) -> Result<String>;

    /// 获取提交历史
    ///
    /// # 参数
    ///
    /// * `path` - Git仓库路径
    /// * `count` - 获取的提交数量
    ///
    /// # 返回值
    ///
    /// * `Ok(String)` - 提交历史输出
    /// * `Err(SyncError)` - 获取历史失败
    fn log(&self, path: &Path, count: Option<usize>) -> Result<String>;

    /// 检查工作目录是否干净
    ///
    /// # 参数
    ///
    /// * `path` - Git仓库路径
    ///
    /// # 返回值
    ///
    /// * `Ok(bool)` - true表示工作目录干净，false表示有未提交的更改
    /// * `Err(SyncError)` - 检查失败
    fn is_clean(&self, path: &Path) -> Result<bool>;
}

// 重新导出具体实现
pub use super::git_provider::{GitOperationsFactory, GitProvider, ProviderType};
pub use super::mock_git::MockGitOperations;
pub use super::real_git::RealGitOperations;
