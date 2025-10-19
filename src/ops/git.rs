use std::path::Path;

use super::git_operations::GitOperations;
use crate::error::Result;

/// 提交 Git 更改（使用自定义Git操作实现）
///
/// 这个函数会先添加所有更改到暂存区，然后提交。
/// 这样可以确保新文件和修改的文件都能被正确提交。
/// 使用GitOperations trait，支持真实Git命令和Mock实现。
///
/// # 参数
///
/// * `git_ops`: Git操作实现对象
/// * `path`: Git 本地目录
/// * `message`: 提交消息
///
/// # 示例
///
/// ```ignore
/// use svn2git::{git_commit_with_ops, RealGitOperations};
/// use std::path::PathBuf;
///
/// let git_ops = RealGitOperations::new();
/// let path = PathBuf::from("/path/to/repo");
/// git_commit_with_ops(&git_ops, &path, "测试提交").expect("提交失败");
/// ```
pub fn git_commit_with_ops<T: GitOperations + ?Sized>(
    git_ops: &T,
    path: &Path,
    message: &str,
) -> Result<()> {
    println!("正在提交 Git 更改");

    // 步骤1: 添加所有更改到暂存区
    git_ops.add_all(path)?;
    println!("已添加所有更改到暂存区");

    // 步骤2: 提交暂存的更改
    git_ops.commit(path, message)?;
    println!("Git 提交成功：{}", message);

    Ok(())
}

/// 使用默认真实Git实现提交更改
///
/// 这是一个便捷函数，使用RealGitOperations作为默认实现
///
/// # 参数
///
/// * `path`: Git 本地目录
/// * `message`: 提交消息
///
/// # 示例
///
/// ```ignore
/// use svn2git::git_commit_real;
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("/path/to/repo");
/// git_commit_real(&path, "测试提交").expect("提交失败");
/// ```
pub fn git_commit_real(path: &Path, message: &str) -> Result<()> {
    let git_ops = super::RealGitOperations::new();
    git_commit_with_ops(&git_ops, path, message)
}
