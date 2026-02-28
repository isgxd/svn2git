use crate::{
    config::{FileStorage, HistoryManager, SyncConfig},
    error::{Result, SyncError},
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

/// 同步运行选项（防事故）
#[derive(Debug, Clone, Default)]
pub struct SyncRunOptions {
    /// 仅预览同步计划，不执行任何写入操作
    pub dry_run: bool,
    /// 最多同步多少条日志（按SVN返回顺序）
    pub limit: Option<usize>,
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
        self.run_with_options(&SyncRunOptions::default())
    }

    /// 按选项执行同步
    pub fn run_with_options(&self, options: &SyncRunOptions) -> Result<()> {
        let mut svn_logs = self.svn_operations.get_logs(&self.config.svn_dir)?;
        svn_logs = limit_logs(svn_logs, options.limit);

        if svn_logs.is_empty() {
            println!("没有可同步的 SVN 日志");
            return Ok(());
        }

        if options.dry_run {
            println!("dry-run 模式：仅预览，不会执行 svn update 或 git commit");
            for log in &svn_logs {
                println!("[预览] r{} -> SVN: {}", log.version, log.message);
            }
            return Ok(());
        }

        if !confirm_sync_with_interactor(&svn_logs, self.interactor.as_ref()) {
            println!("同步已取消");
            return Ok(());
        }

        for (idx, log) in svn_logs.iter().enumerate() {
            println!("准备更新到 SVN 版本：{}", log.version);

            self.svn_operations
                .update_to_rev(&self.config.svn_dir, &log.version)
                .map_err(|e| {
                    SyncError::App(format!(
                        "同步第 {} 条日志失败（SVN r{}）：{}",
                        idx + 1,
                        log.version,
                        e
                    ))
                })?;
            println!("更新完成");

            self.ensure_git_conflict_free().map_err(|e| {
                SyncError::App(format!(
                    "同步第 {} 条日志失败（SVN r{}）：{}",
                    idx + 1,
                    log.version,
                    e
                ))
            })?;

            git_commit_with_ops(
                self.git_operations.as_ref(),
                &self.config.git_dir,
                &format!("SVN: {}", &log.message),
            )
            .map_err(|e| {
                SyncError::App(format!(
                    "同步第 {} 条日志失败（SVN r{}）：{}",
                    idx + 1,
                    log.version,
                    e
                ))
            })?;
            println!("提交到 Git：{}", log.message);
        }

        self.history.save()
    }

    fn ensure_git_conflict_free(&self) -> Result<()> {
        let status = self.git_operations.status(&self.config.git_dir)?;
        if has_conflict_entries(&status) {
            return Err(SyncError::App(
                "检测到 Git 冲突状态（如 UU/AA/DU），已停止后续同步".into(),
            ));
        }
        Ok(())
    }
}

fn limit_logs(logs: Vec<crate::ops::SvnLog>, limit: Option<usize>) -> Vec<crate::ops::SvnLog> {
    match limit {
        Some(n) => logs.into_iter().take(n).collect(),
        None => logs,
    }
}

fn has_conflict_entries(status: &str) -> bool {
    status.lines().any(|line| {
        if line.len() < 2 {
            return false;
        }
        matches!(&line[..2], "DD" | "AU" | "UD" | "UA" | "DU" | "AA" | "UU")
    })
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::Path, path::PathBuf, rc::Rc, str::FromStr};

    use crate::{
        config::{HistoryManager, MockFileStorage, SyncConfig},
        error::SyncError,
        interactor::MockUserInteractor,
        ops::{GitOperations, SvnLog},
    };

    use super::{MockSvnOperations, SyncRunOptions, SyncTool, has_conflict_entries, limit_logs};

    struct TestGitState {
        add_all_calls: usize,
        commit_messages: Vec<String>,
        status_calls: usize,
        status_output: String,
    }

    struct TestGitOperations {
        state: Rc<RefCell<TestGitState>>,
    }

    impl TestGitOperations {
        fn new(status_output: &str) -> (Self, Rc<RefCell<TestGitState>>) {
            let state = Rc::new(RefCell::new(TestGitState {
                add_all_calls: 0,
                commit_messages: Vec::new(),
                status_calls: 0,
                status_output: status_output.to_string(),
            }));
            (
                Self {
                    state: state.clone(),
                },
                state,
            )
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
            let _ = path;
            self.state.borrow_mut().add_all_calls += 1;
            Ok(())
        }

        fn commit(&self, _path: &Path, message: &str) -> crate::error::Result<()> {
            self.state
                .borrow_mut()
                .commit_messages
                .push(message.to_string());
            Ok(())
        }

        fn status(&self, _path: &Path) -> crate::error::Result<String> {
            let mut state = self.state.borrow_mut();
            state.status_calls += 1;
            Ok(state.status_output.clone())
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

        let (git_ops_impl, git_state) = TestGitOperations::new("");
        let git_ops = Box::new(git_ops_impl);
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            git_ops,
            Box::new(svn_ops),
        );

        let result = tool.run();
        assert!(result.is_ok());
        assert_eq!(git_state.borrow().add_all_calls, 2);
        assert_eq!(git_state.borrow().commit_messages.len(), 2);
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

        let (git_ops_impl, git_state) = TestGitOperations::new("");
        let git_ops = Box::new(git_ops_impl);
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            git_ops,
            Box::new(svn_ops),
        );

        let result = tool.run();
        assert!(result.is_ok());
        assert_eq!(git_state.borrow().add_all_calls, 0);
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

        let (git_ops_impl, git_state) = TestGitOperations::new("");
        let git_ops = Box::new(git_ops_impl);
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            git_ops,
            Box::new(svn_ops),
        );

        let result = tool.run();
        assert!(result.is_err());
        assert_eq!(git_state.borrow().add_all_calls, 0);
    }

    #[test]
    fn test_run_dry_run_should_not_update_or_commit_or_save() {
        let config = create_config();
        let history = create_history_manager(0);

        let mut interactor = MockUserInteractor::new();
        interactor.expect_confirm_sync().times(0);

        let mut svn_ops = MockSvnOperations::new();
        svn_ops.expect_get_logs().returning(|_| {
            Ok(vec![SvnLog {
                version: "11".into(),
                message: "dry run".into(),
            }])
        });
        svn_ops.expect_update_to_rev().times(0);

        let (git_ops_impl, git_state) = TestGitOperations::new("");
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            Box::new(git_ops_impl),
            Box::new(svn_ops),
        );

        let result = tool.run_with_options(&SyncRunOptions {
            dry_run: true,
            limit: None,
        });
        assert!(result.is_ok());
        assert_eq!(git_state.borrow().add_all_calls, 0);
        assert_eq!(git_state.borrow().commit_messages.len(), 0);
        assert_eq!(git_state.borrow().status_calls, 0);
    }

    #[test]
    fn test_run_limit_should_only_process_first_n_logs() {
        let config = create_config();
        let history = create_history_manager(1);

        let mut interactor = MockUserInteractor::new();
        interactor.expect_confirm_sync().returning(|_| true);

        let mut svn_ops = MockSvnOperations::new();
        svn_ops.expect_get_logs().returning(|_| {
            Ok(vec![
                SvnLog {
                    version: "1".into(),
                    message: "m1".into(),
                },
                SvnLog {
                    version: "2".into(),
                    message: "m2".into(),
                },
            ])
        });
        svn_ops
            .expect_update_to_rev()
            .times(1)
            .returning(|_, _| Ok(()));

        let (git_ops_impl, git_state) = TestGitOperations::new("");
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            Box::new(git_ops_impl),
            Box::new(svn_ops),
        );

        let result = tool.run_with_options(&SyncRunOptions {
            dry_run: false,
            limit: Some(1),
        });
        assert!(result.is_ok());
        assert_eq!(git_state.borrow().add_all_calls, 1);
        assert_eq!(git_state.borrow().commit_messages, vec!["SVN: m1"]);
    }

    #[test]
    fn test_run_should_stop_when_git_conflict_detected() {
        let config = create_config();
        let history = create_history_manager(0);

        let mut interactor = MockUserInteractor::new();
        interactor.expect_confirm_sync().returning(|_| true);

        let mut svn_ops = MockSvnOperations::new();
        svn_ops.expect_get_logs().returning(|_| {
            Ok(vec![SvnLog {
                version: "5".into(),
                message: "conflict".into(),
            }])
        });
        svn_ops
            .expect_update_to_rev()
            .times(1)
            .returning(|_, _| Ok(()));

        let (git_ops_impl, git_state) = TestGitOperations::new("UU conflict.txt");
        let tool = SyncTool::with_svn_operations(
            config,
            history,
            Box::new(interactor),
            Box::new(git_ops_impl),
            Box::new(svn_ops),
        );

        let result = tool.run();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("检测到 Git 冲突状态"));
        assert_eq!(git_state.borrow().status_calls, 1);
        assert_eq!(git_state.borrow().add_all_calls, 0);
    }

    #[test]
    fn test_has_conflict_entries() {
        assert!(has_conflict_entries("UU file.txt"));
        assert!(has_conflict_entries("AA file.txt"));
        assert!(!has_conflict_entries("?? file.txt\n M file2.txt"));
    }

    #[test]
    fn test_limit_logs() {
        let logs = vec![
            SvnLog {
                version: "1".into(),
                message: "a".into(),
            },
            SvnLog {
                version: "2".into(),
                message: "b".into(),
            },
        ];
        let limited = limit_logs(logs, Some(1));
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].version, "1");
    }
}
