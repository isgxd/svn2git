use crate::{
    config::{FileStorage, HistoryManager, SyncConfig},
    error::Result,
    interactor,
    ops::{get_svn_logs, git_commit, svn_update_to_rev},
};

/// 同步工具
pub struct SyncTool<S: FileStorage> {
    config: SyncConfig,
    history: HistoryManager<S>,
}

impl<S: FileStorage> SyncTool<S> {
    /// 创建一个新的同步工具
    pub fn new(config: SyncConfig, history: HistoryManager<S>) -> Self {
        Self { config, history }
    }

    /// 执行同步
    pub fn run(&self) -> Result<()> {
        let svn_logs = get_svn_logs(&self.config.svn_dir)?;

        if !interactor::confirm_sync(&svn_logs) {
            println!("同步已取消");
            return Ok(());
        }

        for log in svn_logs.iter() {
            println!("准备更新到 SVN 版本：{}", log.version);

            svn_update_to_rev(&self.config.svn_dir, &log.version)?;
            println!("更新完成");

            git_commit(&self.config.git_dir, &format!("SVN: {}", &log.message))?;
            println!("提交到 Git：{}", log.message);
        }

        self.history.save()
    }
}
