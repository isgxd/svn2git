use crate::{
    config::{FileStorage, HistoryManager, SyncConfig},
    error::Result,
    interactor::{UserInteractor, confirm_sync_with_interactor},
    ops::{GitOperations, get_svn_logs, git_commit_with_ops, svn_update_to_rev},
};

/// 同步工具
pub struct SyncTool<S: FileStorage> {
    config: SyncConfig,
    history: HistoryManager<S>,
    interactor: Box<dyn UserInteractor>,
    git_operations: Box<dyn GitOperations>,
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
        Self {
            config,
            history,
            interactor,
            git_operations,
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
        let svn_logs = get_svn_logs(&self.config.svn_dir)?;

        if !confirm_sync_with_interactor(&svn_logs, self.interactor.as_ref()) {
            println!("同步已取消");
            return Ok(());
        }

        for log in svn_logs.iter() {
            println!("准备更新到 SVN 版本：{}", log.version);

            svn_update_to_rev(&self.config.svn_dir, &log.version)?;
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
