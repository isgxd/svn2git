//! Mock SVN操作模块
//!
//! 提供SVN操作的Mock实现，用于单元测试，避免依赖真实的SVN命令

use crate::error::{Result, SyncError};

/// Mock SVN仓库
///
/// 在内存中模拟SVN仓库的状态和操作，用于测试
#[derive(Debug, Clone)]
pub struct MockSvnRepo {
    /// 仓库路径
    pub path: std::path::PathBuf,
    /// 是否已初始化
    initialized: bool,
}

impl MockSvnRepo {
    /// 创建新的Mock SVN仓库
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径
    ///
    /// # 返回值
    ///
    /// 返回新的MockSvnRepo实例
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            initialized: false,
        }
    }

    /// 初始化SVN仓库
    pub fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Err(SyncError::App("SVN仓库已经初始化".to_string()));
        }
        self.initialized = true;
        Ok(())
    }

    /// 检查仓库是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_svn_repo_creation() {
        let repo = MockSvnRepo::new("/test".into());
        assert!(!repo.is_initialized());
    }

    #[test]
    fn test_mock_svn_repo_init() {
        let mut repo = MockSvnRepo::new("/test".into());
        assert!(repo.init().is_ok());
        assert!(repo.is_initialized());
        assert!(repo.init().is_err());
    }
}
