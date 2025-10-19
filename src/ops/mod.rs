mod git;
mod git_operations;
mod git_provider;
mod mock_git;
mod real_git;
mod svn;

// Git操作抽象和实现
pub use git_operations::{
    GitOperations, GitOperationsFactory, GitProvider, MockGitOperations, ProviderType,
    RealGitOperations,
};

// Git操作函数（只导出公共API）
pub use git::{git_commit_real, git_commit_with_ops};

// SVN操作
pub use svn::*;
