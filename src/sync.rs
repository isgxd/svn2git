use crate::{
    config::{FileStorage, HistoryManager, SyncConfig},
    error::Result,
    interactor::{UserInteractor, confirm_sync_with_interactor},
    ops::{GitOperations, get_svn_logs, git_commit_with_ops, svn_update_to_rev},
};

/// SVN操作抽象接口
#[cfg_attr(test, mockall::automock)]
pub trait SvnOperations {
    fn get_logs(&self, path: &std::path::Path) -> Result<Vec<crate::ops::SvnLog>>;
    fn update_to_rev(&self, path: &std::path::Path, rev: &str) -> Result<()>;
}

/// 真实SVN操作实现
pub struct RealSvnOperations;

impl SvnOperations for RealSvnOperations {
    fn get_logs(&self, path: &std::path::Path) -> Result<Vec<crate::ops::SvnLog>> {
        get_svn_logs(&path.to_path_buf())
    }

    fn update_to_rev(&self, path: &std::path::Path, rev: &str) -> Result<()> {
        svn_update_to_rev(&path.to_path_buf(), rev)
    }
}

/// 同步工具
pub struct SyncTool<S: FileStorage> {
    config: SyncConfig,
    history: HistoryManager<S>,
    interactor: Box<dyn UserInteractor>,
    git_operations: Box<dyn GitOperations>,
    svn_operations: Box<dyn SvnOperations>,
}

impl<S: FileStorage> SyncTool<S> {
    /// 创建一个新的同步工具
    ///
    /// # 参数
    ///
    /// * `config` - 同步配置
    /// * `history` - 历史记录管理器
    /// * `interactor` - 用户交互器
    /// * `git_operations` - Git操作实现
    pub fn new(
        config: SyncConfig,
        history: HistoryManager<S>,
        interactor: Box<dyn UserInteractor>,
        git_operations: Box<dyn GitOperations>,
    ) -> Self {
        Self::with_svn_operations(
            config,
            history,
            interactor,
            git_operations,
            Box::new(RealSvnOperations),
        )
    }

    /// 创建自定义SVN实现的同步工具
    pub fn with_svn_operations(
        config: SyncConfig,
        history: HistoryManager<S>,
        interactor: Box<dyn UserInteractor>,
        git_operations: Box<dyn GitOperations>,
        svn_operations: Box<dyn SvnOperations>,
    ) -> Self {
        Self {
            config,
            history,
            interactor,
            git_operations,
            svn_operations,
        }
    }

    /// 创建使用默认真实Git实现的同步工具
    ///
    /// 这是一个便捷方法，创建使用RealGitOperations的SyncTool
    ///
    /// # 参数
    ///
    /// * `config` - 同步配置
    /// * `history` - 历史记录管理器
    /// * `interactor` - 用户交互器
    pub fn with_real_git(
        config: SyncConfig,
        history: HistoryManager<S>,
        interactor: Box<dyn UserInteractor>,
    ) -> Self {
        use super::RealGitOperations;
        let git_operations = Box::new(RealGitOperations::new());
        Self::new(config, history, interactor, git_operations)
    }

    /// 执行同步
    pub fn run(&self) -> Result<()> {
        let svn_logs = self.svn_operations.get_logs(&self.config.svn_dir)?;

        if !confirm_sync_with_interactor(&svn_logs, self.interactor.as_ref()) {
            println!("同步已取消");
            return Ok(());
        }

        for log in svn_logs.iter() {
            println!("准备更新到 SVN 版本：{}", log.version);

            self.svn_operations
                .update_to_rev(&self.config.svn_dir, &log.version)?;
            println!("更新完成");

            git_commit_with_ops(
                self.git_operations.as_ref(),
                &self.config.git_dir,
                &format!("SVN: {}", &log.message),
            )?;
            println!("提交到 Git：{}", log.message);
        }

        self.history.save()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::Path, path::PathBuf, str::FromStr};

    use crate::{
        config::{HistoryManager, MockFileStorage, SyncConfig},
        error::SyncError,
        interactor::MockUserInteractor,
        ops::{GitOperations, SvnLog},
    };

    use super::{MockSvnOperations, SyncTool};

    struct TestGitOperations {
        add_all_calls: RefCell<Vec<PathBuf>>,
        commit_messages: RefCell<Vec<String>>,
    }

    impl TestGitOperations {
        fn new() -> Self {
            Self {
                add_all_calls: RefCell::new(Vec::new()),
                commit_messages: RefCell::new(Vec::new()),
            }
        }
    }

    impl GitOperations for TestGitOperations {
        fn init(&self, _path: &Path) -> crate::error::Result<()> {
            Ok(())
        }

        fn config_user(&self, _path: &Path, _name: &str, _email: &str) -> crate::error::Result<()> {
            Ok(())
        }

        fn add_all(&self, path: &Path) -> crate::error::Result<()> {
            self.add_all_calls.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn commit(&self, _path: &Path, message: &str) -> crate::error::Result<()> {
            self.commit_messages.borrow_mut().push(message.to_string());
            Ok(())
        }

        fn status(&self, _path: &Path) -> crate::error::Result<String> {
            Ok(String::new())
        }

        fn log(&self, _path: &Path, _count: Option<usize>) -> crate::error::Result<String> {
            Ok(String::new())
        }

        fn is_clean(&self, _path: &Path) -> crate::error::Result<bool> {
            Ok(true)
        }
    }

    fn create_history_manager(expect_save_count: usize) -> HistoryManager<MockFileStorage> {
        let mut storage = MockFileStorage::new();
        storage.expect_load().returning(|| Ok(vec![]));
        if expect_save_count > 0 {
            storage
                .expect_save()
                .times(expect_save_count)
                .returning(|_| Ok(()));
        }
        HistoryManager::new(storage).unwrap()
    }

    fn create_config() -> SyncConfig {
        SyncConfig::new(
            PathBuf::from_str("svn_dir").unwrap(),
            PathBuf::from_str("git_dir").unwrap(),
        )
    }

    #[test]
    fn test_run_success_with_mock_svn_and_git() {
        let config = create_config();
        let history = create_history_manager(1);

        let mut interactor = MockUserInteractor::new();
        interactor.expect_confirm_sync().returning(|_| true);

        let mut svn_ops = MockSvnOperations::new();
        svn_ops.expect_get_logs().returning(|_| {
            Ok(vec![
                SvnLog {
                    version: "1".into(),
                    message: "初始提交".into(),
                },
                SvnLog {
                    version: "2".into(),
                    message: "修复问题".into(),
                },
            ])
        });
        svn_ops
            .expect_update_to_rev()
            .times(2)
            .returning(|_, _| Ok(()));

        let git_ops = Box::new(TestGitOperations::new());
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            git_ops,
            Box::new(svn_ops),
        );

        let result = tool.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_cancel_should_not_update_or_save() {
        let config = create_config();
        let history = create_history_manager(0);

        let mut interactor = MockUserInteractor::new();
        interactor.expect_confirm_sync().returning(|_| false);

        let mut svn_ops = MockSvnOperations::new();
        svn_ops.expect_get_logs().returning(|_| {
            Ok(vec![SvnLog {
                version: "10".into(),
                message: "测试".into(),
            }])
        });
        svn_ops.expect_update_to_rev().times(0);

        let git_ops = Box::new(TestGitOperations::new());
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            git_ops,
            Box::new(svn_ops),
        );

        let result = tool.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_should_return_error_when_svn_update_fails() {
        let config = create_config();
        let history = create_history_manager(0);

        let mut interactor = MockUserInteractor::new();
        interactor.expect_confirm_sync().returning(|_| true);

        let mut svn_ops = MockSvnOperations::new();
        svn_ops.expect_get_logs().returning(|_| {
            Ok(vec![SvnLog {
                version: "3".into(),
                message: "触发失败".into(),
            }])
        });
        svn_ops
            .expect_update_to_rev()
            .times(1)
            .returning(|_, _| Err(SyncError::App("svn update failed".into())));

        let git_ops = Box::new(TestGitOperations::new());
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            git_ops,
            Box::new(svn_ops),
        );

        let result = tool.run();
        assert!(result.is_err());
    }
}
