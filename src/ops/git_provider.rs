//! Git提供者模块
//!
//! 提供统一的Git操作抽象，支持真实Git和Mock实现的无缝切换

use std::path::Path;

use super::git_operations::{GitOperations, RealGitOperations};
use super::mock_git::MockGitOperations;

/// Git提供者类型
///
/// 支持真实Git操作和Mock操作两种实现方式
#[derive(Debug, Clone)]
pub enum GitProvider {
    /// 真实Git操作实现
    Real(RealGitOperations),
    /// Mock Git操作实现（用于测试）
    Mock(MockGitOperations),
}

impl GitProvider {
    /// 创建新的Git提供者实例
    ///
    /// # 参数
    ///
    /// * `provider_type` - 提供者类型
    ///
    /// # 返回值
    ///
    /// 返回相应的Git提供者实例
    ///
    /// # 示例
    ///
    /// ```
    /// use svn2git::{GitProvider, ProviderType};
    ///
    /// // 创建真实Git提供者
    /// let real_provider = GitProvider::new(ProviderType::Real);
    ///
    /// // 创建Mock Git提供者
    /// let mock_provider = GitProvider::new(ProviderType::Mock);
    /// ```
    pub fn new(provider_type: ProviderType) -> Self {
        match provider_type {
            ProviderType::Real => Self::Real(RealGitOperations::new()),
            ProviderType::Mock => Self::Mock(MockGitOperations::new()),
        }
    }

    /// 根据环境自动创建Git提供者
    ///
    /// 在测试环境中使用Mock实现，生产环境使用真实实现
    ///
    /// # 返回值
    ///
    /// 返回自动选择的Git提供者实例
    pub fn auto() -> Self {
        let provider_type = if cfg!(test) {
            ProviderType::Mock
        } else {
            ProviderType::Real
        };
        Self::new(provider_type)
    }
}

impl GitOperations for GitProvider {
    fn init(&self, path: &Path) -> crate::error::Result<()> {
        match self {
            GitProvider::Real(ops) => ops.init(path),
            GitProvider::Mock(ops) => ops.init(path),
        }
    }

    fn config_user(&self, path: &Path, name: &str, email: &str) -> crate::error::Result<()> {
        match self {
            GitProvider::Real(ops) => ops.config_user(path, name, email),
            GitProvider::Mock(ops) => ops.config_user(path, name, email),
        }
    }

    fn add_all(&self, path: &Path) -> crate::error::Result<()> {
        match self {
            GitProvider::Real(ops) => ops.add_all(path),
            GitProvider::Mock(ops) => ops.add_all(path),
        }
    }

    fn commit(&self, path: &Path, message: &str) -> crate::error::Result<()> {
        match self {
            GitProvider::Real(ops) => ops.commit(path, message),
            GitProvider::Mock(ops) => ops.commit(path, message),
        }
    }

    fn status(&self, path: &Path) -> crate::error::Result<String> {
        match self {
            GitProvider::Real(ops) => ops.status(path),
            GitProvider::Mock(ops) => ops.status(path),
        }
    }

    fn log(&self, path: &Path, count: Option<usize>) -> crate::error::Result<String> {
        match self {
            GitProvider::Real(ops) => ops.log(path, count),
            GitProvider::Mock(ops) => ops.log(path, count),
        }
    }

    fn is_clean(&self, path: &Path) -> crate::error::Result<bool> {
        match self {
            GitProvider::Real(ops) => ops.is_clean(path),
            GitProvider::Mock(ops) => ops.is_clean(path),
        }
    }
}

/// Git提供者类型枚举
///
/// 用于指定使用哪种Git操作实现
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderType {
    /// 使用真实的Git命令
    Real,
    /// 使用Mock实现（用于测试）
    Mock,
}

/// Git操作工厂
///
/// 提供创建不同Git操作实现的统一接口
pub struct GitOperationsFactory;

impl GitOperationsFactory {
    /// 根据提供者类型创建Git操作实例
    ///
    /// # 参数
    ///
    /// * `provider_type` - 提供者类型
    ///
    /// # 返回值
    ///
    /// 返回相应的Git操作实例
    pub fn create(provider_type: ProviderType) -> GitProvider {
        GitProvider::new(provider_type)
    }

    /// 根据字符串创建Git操作实例
    ///
    /// # 参数
    ///
    /// * `type_str` - 提供者类型字符串 ("real" 或 "mock")
    ///
    /// # 返回值
    ///
    /// * `Ok(GitProvider)` - 创建成功
    /// * `Err(String)` - 无效的类型字符串
    pub fn create_from_string(type_str: &str) -> Result<GitProvider, String> {
        match type_str.to_lowercase().as_str() {
            "real" => Ok(GitProvider::new(ProviderType::Real)),
            "mock" => Ok(GitProvider::new(ProviderType::Mock)),
            _ => Err(format!(
                "无效的Git提供者类型: {}。支持的类型: real, mock",
                type_str
            )),
        }
    }

    /// 根据环境变量创建Git操作实例
    ///
    /// 从环境变量 `SVN2GIT_GIT_PROVIDER` 读取提供者类型
    ///
    /// 如果环境变量未设置，则使用默认的Real实现
    ///
    /// # 返回值
    ///
    /// 返回相应或默认的Git操作实例
    pub fn create_from_env() -> GitProvider {
        match std::env::var("SVN2GIT_GIT_PROVIDER") {
            Ok(type_str) => Self::create_from_string(&type_str).unwrap_or_else(|_| {
                eprintln!(
                    "警告: 无效的Git提供者类型 '{}', 使用默认的Real实现",
                    type_str
                );
                GitProvider::new(ProviderType::Real)
            }),
            Err(_) => GitProvider::new(ProviderType::Real),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_git_provider_creation() {
        let real_provider = GitProvider::new(ProviderType::Real);
        let mock_provider = GitProvider::new(ProviderType::Mock);

        // 验证提供者创建成功
        match real_provider {
            GitProvider::Real(_) => {} // 期望的类型
            _ => panic!("期望创建Real提供者"),
        }

        match mock_provider {
            GitProvider::Mock(_) => {} // 期望的类型
            _ => panic!("期望创建Mock提供者"),
        }
    }

    #[test]
    fn test_factory_create() {
        let real_provider = GitOperationsFactory::create(ProviderType::Real);
        let mock_provider = GitOperationsFactory::create(ProviderType::Mock);

        // 验证工厂创建成功
        assert!(matches!(real_provider, GitProvider::Real(_)));
        assert!(matches!(mock_provider, GitProvider::Mock(_)));
    }

    #[test]
    fn test_factory_create_from_string() {
        // 测试有效输入
        let real_result = GitOperationsFactory::create_from_string("real");
        let mock_result = GitOperationsFactory::create_from_string("mock");
        let upper_result = GitOperationsFactory::create_from_string("REAL"); // 测试大小写不敏感

        assert!(real_result.is_ok());
        assert!(mock_result.is_ok());
        assert!(upper_result.is_ok());

        assert!(matches!(real_result.unwrap(), GitProvider::Real(_)));
        assert!(matches!(mock_result.unwrap(), GitProvider::Mock(_)));
        assert!(matches!(upper_result.unwrap(), GitProvider::Real(_)));

        // 测试无效输入
        let invalid_result = GitOperationsFactory::create_from_string("invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_mock_git_operations() {
        let mock_provider = GitProvider::new(ProviderType::Mock);
        let test_path = PathBuf::from("/test/repo");

        // 测试Mock操作
        let init_result = mock_provider.init(&test_path);
        assert!(init_result.is_ok(), "Mock Git初始化应该成功");

        let config_result = mock_provider.config_user(&test_path, "测试用户", "test@example.com");
        assert!(config_result.is_ok(), "Mock Git用户配置应该成功");

        let status_result = mock_provider.status(&test_path);
        assert!(status_result.is_ok(), "Mock Git状态查询应该成功");
    }

    #[test]
    fn test_provider_type_equality() {
        assert_eq!(ProviderType::Real, ProviderType::Real);
        assert_eq!(ProviderType::Mock, ProviderType::Mock);
        assert_ne!(ProviderType::Real, ProviderType::Mock);
    }
}
