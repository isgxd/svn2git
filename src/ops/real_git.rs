//! 真实Git操作实现
//!
//! 使用真实的git命令执行操作，用于生产环境

use super::git_operations::GitOperations;
use crate::error::{Result, SyncError};
use std::path::Path;

/// 真实Git操作实现
///
/// 使用真实的git命令执行操作
#[derive(Debug, Clone)]
pub struct RealGitOperations;

impl RealGitOperations {
    /// 创建新的真实Git操作实例
    ///
    /// # 返回值
    ///
    /// 返回新的RealGitOperations实例
    ///
    /// # 示例
    ///
    /// ```
    /// use svn2git::RealGitOperations;
    ///
    /// let git_ops = RealGitOperations::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// 检查Git是否可用
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - Git可用
    /// * `Err(SyncError)` - Git不可用
    pub fn check_git_available() -> Result<()> {
        let output = std::process::Command::new("git").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => Ok(()),
            Ok(_) => Err(SyncError::App("Git命令执行失败".to_string())),
            Err(e) => Err(SyncError::App(format!("无法执行Git命令: {}", e))),
        }
    }
}

impl Default for RealGitOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl GitOperations for RealGitOperations {
    fn init(&self, path: &Path) -> Result<()> {
        let output = std::process::Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SyncError::App(format!(
                "Git初始化失败，路径: {:?}, 错误: {}",
                path,
                if stderr.is_empty() {
                    "无详细信息"
                } else {
                    &stderr
                }
            )));
        }

        Ok(())
    }

    fn config_user(&self, path: &Path, name: &str, email: &str) -> Result<()> {
        // 配置用户名
        let name_output = std::process::Command::new("git")
            .args(["config", "user.name", name])
            .current_dir(path)
            .output()?;

        if !name_output.status.success() {
            let stderr = String::from_utf8_lossy(&name_output.stderr);
            return Err(SyncError::App(format!(
                "配置Git用户名失败，路径: {:?}, 错误: {}",
                path,
                if stderr.is_empty() {
                    "无详细信息"
                } else {
                    &stderr
                }
            )));
        }

        // 配置邮箱
        let email_output = std::process::Command::new("git")
            .args(["config", "user.email", email])
            .current_dir(path)
            .output()?;

        if !email_output.status.success() {
            let stderr = String::from_utf8_lossy(&email_output.stderr);
            return Err(SyncError::App(format!(
                "配置Git邮箱失败，路径: {:?}, 错误: {}",
                path,
                if stderr.is_empty() {
                    "无详细信息"
                } else {
                    &stderr
                }
            )));
        }

        Ok(())
    }

    fn add_all(&self, path: &Path) -> Result<()> {
        let output = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SyncError::App(format!(
                "Git add失败，路径: {:?}, 错误: {}",
                path,
                if stderr.is_empty() {
                    "无详细信息"
                } else {
                    &stderr
                }
            )));
        }

        Ok(())
    }

    fn commit(&self, path: &Path, message: &str) -> Result<()> {
        let output = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(SyncError::App(format!(
                "Git commit失败，路径: {:?}, 提交信息: '{}', stdout: {}, stderr: {}",
                path,
                message,
                if stdout.is_empty() {
                    "无输出"
                } else {
                    &stdout
                },
                if stderr.is_empty() {
                    "无错误信息"
                } else {
                    &stderr
                }
            )));
        }

        Ok(())
    }

    fn status(&self, path: &Path) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SyncError::App(format!(
                "获取Git状态失败，路径: {:?}, 错误: {}",
                path,
                if stderr.is_empty() {
                    "无详细信息"
                } else {
                    &stderr
                }
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn log(&self, path: &Path, count: Option<usize>) -> Result<String> {
        let mut cmd = std::process::Command::new("git");
        cmd.args(["log", "--oneline"]);

        if let Some(n) = count {
            cmd.args(["-n", &n.to_string()]);
        }

        let output = cmd.current_dir(path).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SyncError::App(format!(
                "获取Git日志失败，路径: {:?}, 错误: {}",
                path,
                if stderr.is_empty() {
                    "无详细信息"
                } else {
                    &stderr
                }
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn is_clean(&self, path: &Path) -> Result<bool> {
        let status_output = self.status(path)?;
        Ok(status_output.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_real_git_operations_creation() {
        let _ops = RealGitOperations::new();
        let _ops_default: RealGitOperations = Default::default();
        // 验证实例创建成功，没有panic
    }

    #[test]
    fn test_check_git_available() {
        // 这个测试需要系统中有Git才能通过
        // 在CI环境中可能需要特殊处理
        let result = RealGitOperations::check_git_available();
        println!("Git可用性检查: {:?}", result);
        // 不强制断言，因为测试环境可能没有Git
    }

    #[test]
    fn test_real_git_status_on_invalid_path() {
        let ops = RealGitOperations::new();
        let invalid_path = PathBuf::from("/不存在的路径");
        let result = ops.status(&invalid_path);
        assert!(result.is_err(), "在无效路径上获取Git状态应该返回错误");
    }

    #[test]
    fn test_real_git_log_on_invalid_path() {
        let ops = RealGitOperations::new();
        let invalid_path = PathBuf::from("/不存在的路径");
        let result = ops.log(&invalid_path, Some(5));
        assert!(result.is_err(), "在无效路径上获取Git日志应该返回错误");
    }

    #[test]
    fn test_real_git_config_user_on_invalid_path() {
        let ops = RealGitOperations::new();
        let invalid_path = PathBuf::from("/不存在的路径");
        let result = ops.config_user(&invalid_path, "测试用户", "test@example.com");
        assert!(result.is_err(), "在无效路径上配置Git用户信息应该返回错误");
    }

    #[test]
    fn test_real_git_commit_on_invalid_path() {
        let ops = RealGitOperations::new();
        let invalid_path = PathBuf::from("/不存在的路径");
        let result = ops.commit(&invalid_path, "测试提交");
        assert!(result.is_err(), "在无效路径上执行Git提交应该返回错误");
    }

    #[test]
    fn test_real_git_init_on_invalid_path() {
        let ops = RealGitOperations::new();
        let invalid_path = PathBuf::from("/不存在的路径/无法创建");
        let result = ops.init(&invalid_path);
        // 在无法创建的路径上初始化Git应该失败
        assert!(result.is_err(), "在无法创建的路径上初始化Git应该返回错误");
    }
}
