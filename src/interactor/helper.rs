use crate::{
    config::{DiskStorage, FileStorage, HistoryManager, SyncConfig},
    error::Result,
    interactor::{DefaultUserInteractor, UserInteractor},
    ops::SvnLog,
};

use std::{path::PathBuf, str::FromStr};

/// 选择或创建配置
///
/// # 参数
///
/// * `svn_dir`: SVN 本地目录
/// * `git_dir`: Git 本地目录
/// * `history`: 历史记录
pub fn select_or_create_config_default(
    svn_dir: Option<PathBuf>,
    git_dir: Option<PathBuf>,
    history: &mut HistoryManager<DiskStorage>,
) -> Result<SyncConfig> {
    let interactor = DefaultUserInteractor;
    select_or_create_config(svn_dir, git_dir, history, &interactor)
}

/// 确认是否同步
///
/// # 参数
///
/// * `svn_logs`: SVN 日志列表
/// * `git_log`: Git 日志
///
/// # 返回
///
/// 是否同步
pub fn confirm_sync(svn_logs: &[SvnLog]) -> bool {
    let interactor = DefaultUserInteractor;
    interactor.confirm_sync(svn_logs)
}

/// 选择或创建配置
///
/// # 参数
///
/// * `svn_dir`: SVN 本地目录
/// * `git_dir`: Git 本地目录
/// * `history`: 历史记录
/// * `interactor`: 用户交互器
///
/// # 返回
///
/// 同步配置
fn select_or_create_config<S: FileStorage>(
    svn_dir: Option<PathBuf>,
    git_dir: Option<PathBuf>,
    history: &mut HistoryManager<S>,
    interactor: &dyn UserInteractor,
) -> Result<SyncConfig> {
    let config = match (svn_dir, git_dir) {
        (Some(svn), Some(git)) => SyncConfig::new(svn, git),
        _ => {
            if !history.is_empty() {
                let selection = interactor.select_history_record(history.records())?;
                let record = &history.records()[selection];
                record.to_sync_config()
            } else {
                let svn = interactor.input_svn_dir()?;
                let mut git = interactor.input_git_dir()?;

                if git.is_empty() {
                    println!("未输入 Git 文件夹，将使用 SVN 文件夹");
                    git = svn.clone();
                }

                SyncConfig::new(
                    PathBuf::from_str(&svn).unwrap(),
                    PathBuf::from_str(&git).unwrap(),
                )
            }
        }
    };

    history.add_record(config.svn_dir.clone(), config.git_dir.clone());
    history.save()?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use crate::{config::MockFileStorage, interactor::MockUserInteractor};

    use super::*;

    #[test]
    fn test_select_or_create_config() {
        let mut storage = MockFileStorage::new();
        storage.expect_save().returning(|_| Ok(()));
        storage.expect_load().returning(|| Ok(vec![]));

        let mut history = HistoryManager::new(storage).unwrap();

        let mut interactor = MockUserInteractor::new();
        interactor
            .expect_input_svn_dir()
            .returning(|| Ok("s".into()));
        interactor
            .expect_input_git_dir()
            .returning(|| Ok("".into()));

        let config = select_or_create_config(None, None, &mut history, &interactor).unwrap();
        assert_eq!(config.svn_dir, PathBuf::from_str("s").unwrap());
        assert_eq!(config.git_dir, PathBuf::from_str("s").unwrap());
    }
}
