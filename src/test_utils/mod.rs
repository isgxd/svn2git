//! 测试工具模块
//!
//! 提供用于单元测试的Mock工具和辅助函数，避免测试依赖外部的SVN和Git命令

pub mod mock_svn;
pub mod test_factories;

// 重新导出常用的测试工具
pub use mock_svn::*;
pub use test_factories::{GitTestHelper, TestData, TestFactory};

// 重新导出统一Mock实现
pub use crate::ops::{GitProvider, MockGitOperations, ProviderType};

use std::path::PathBuf;

/// 测试用的通用辅助函数
pub struct TestHelpers;

impl TestHelpers {
    /// 创建测试用的临时路径
    ///
    /// # 参数
    ///
    /// * `components` - 路径组件
    ///
    /// # 返回值
    ///
    /// 返回拼接后的路径
    ///
    /// # 示例
    ///
    /// ```
    /// use svn2git::test_utils::TestHelpers;
    ///
    /// let path = TestHelpers::test_path(&["repo", "git"]);
    /// ```
    pub fn test_path(components: &[&str]) -> PathBuf {
        let mut path = PathBuf::from("/test");
        for component in components {
            path.push(component);
        }
        path
    }

    /// 创建测试用的提交消息
    ///
    /// # 参数
    ///
    /// * `index` - 提交索引
    ///
    /// # 返回值
    ///
    /// 返回格式化的提交消息
    pub fn test_commit_message(index: usize) -> String {
        format!("测试提交 #{}", index)
    }

    /// 创建测试用的文件内容
    ///
    /// # 参数
    ///
    /// * `filename` - 文件名
    ///
    /// # 返回值
    ///
    /// 返回测试文件内容
    pub fn test_file_content(filename: &str) -> String {
        format!("这是测试文件 {} 的内容\n测试时间: 2024-07-27", filename)
    }

    /// 创建Mock Git提供者
    ///
    /// 这是一个便捷方法，用于创建Mock Git提供者
    ///
    /// # 返回值
    ///
    /// 返回Mock Git提供者实例
    pub fn create_mock_git_provider() -> GitProvider {
        GitProvider::new(ProviderType::Mock)
    }

    /// 创建真实Git提供者
    ///
    /// 这是一个便捷方法，用于创建真实Git提供者
    ///
    /// # 返回值
    ///
    /// 返回真实Git提供者实例
    pub fn create_real_git_provider() -> GitProvider {
        GitProvider::new(ProviderType::Real)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_helpers_create_path() {
        let path = TestHelpers::test_path(&["svn", "repo"]);
        let path_str = path.to_str().unwrap();
        // 在Windows上路径分隔符是反斜杠，所以我们检查路径组件而不是完整字符串
        assert!(path_str.contains("svn"));
        assert!(path_str.contains("repo"));
        assert!(path_str.starts_with("/test"));
    }

    #[test]
    fn test_test_helpers_commit_message() {
        let message = TestHelpers::test_commit_message(42);
        assert_eq!(message, "测试提交 #42");
    }

    #[test]
    fn test_test_helpers_file_content() {
        let content = TestHelpers::test_file_content("test.txt");
        assert!(content.contains("test.txt"));
        assert!(content.contains("2024-07-27"));
    }
}
