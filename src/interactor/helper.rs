use crate::{
    config::{DiskStorage, FileStorage, HistoryManager, SyncConfig},
    error::Result,
    interactor::{DefaultUserInteractor, UserInteractor},
    ops::SvnLog,
};

use std::{path::PathBuf, str::FromStr};

/// 选择或创建配置（使用默认用户交互器）
///
/// # 参数
///
/// * `svn_dir`: SVN 本地目录
/// * `git_dir`: Git 本地目录
/// * `history`: 历史记录
#[deprecated(note = "使用 select_or_create_config_with_interactor 以获得更好的可测试性")]
pub fn select_or_create_config_default(
    svn_dir: Option<PathBuf>,
    git_dir: Option<PathBuf>,
    history: &mut HistoryManager<DiskStorage>,
) -> Result<SyncConfig> {
    let interactor = DefaultUserInteractor;
    select_or_create_config(svn_dir, git_dir, history, &interactor)
}

/// 选择或创建配置（使用自定义用户交互器）
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
///
/// # 示例
///
/// ```ignore
/// use svn2git::{select_or_create_config_with_interactor, HistoryManager, SyncConfig};
/// use svn2git::{UserInteractor, TestUserInteractor};
/// use std::path::PathBuf;
///
/// let mut history = HistoryManager::new();
/// let interactor = TestUserInteractor::new();
/// let config = select_or_create_config_with_interactor(
///     Some(PathBuf::from("svn")),
///     Some(PathBuf::from("git")),
///     &mut history,
///     &interactor
/// )?;
/// ```
pub fn select_or_create_config_with_interactor<S: FileStorage>(
    svn_dir: Option<PathBuf>,
    git_dir: Option<PathBuf>,
    history: &mut HistoryManager<S>,
    interactor: &dyn UserInteractor,
) -> Result<SyncConfig> {
    select_or_create_config(svn_dir, git_dir, history, interactor)
}

/// 确认是否同步（使用默认用户交互器）
///
/// # 参数
///
/// * `svn_logs`: SVN 日志列表
///
/// # 返回
///
/// 是否同步
#[deprecated(note = "使用 confirm_sync_with_interactor 以获得更好的可测试性")]
pub fn confirm_sync(svn_logs: &[SvnLog]) -> bool {
    let interactor = DefaultUserInteractor;
    interactor.confirm_sync(svn_logs)
}

/// 确认是否同步（使用自定义用户交互器）
///
/// # 参数
///
/// * `svn_logs`: SVN 日志列表
/// * `interactor`: 用户交互器
///
/// # 返回
///
/// 是否同步
///
/// # 示例
///
/// ```ignore
/// use svn2git::{confirm_sync_with_interactor, TestUserInteractor, SvnLog};
/// use svn2git::UserInteractor;
///
/// let interactor = TestUserInteractor::new().with_confirm_result(true);
/// let svn_logs = vec![SvnLog {
///     version: "1".into(),
///     message: "测试提交".into(),
/// }];
///
/// let should_sync = confirm_sync_with_interactor(&svn_logs, &interactor);
/// assert!(should_sync);
/// ```
pub fn confirm_sync_with_interactor(svn_logs: &[SvnLog], interactor: &dyn UserInteractor) -> bool {
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
    use std::path::PathBuf;

    use crate::{
        config::{HistoryRecord, MockFileStorage},
        interactor::MockUserInteractor,
    };

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

    #[test]
    fn test_select_or_create_config_with_cli_paths_should_not_require_input() {
        let mut storage = MockFileStorage::new();
        storage.expect_load().returning(|| Ok(vec![]));
        storage.expect_save().returning(|_| Ok(()));
        let mut history = HistoryManager::new(storage).unwrap();

        let mut interactor = MockUserInteractor::new();
        interactor.expect_input_svn_dir().times(0);
        interactor.expect_input_git_dir().times(0);
        interactor.expect_select_history_record().times(0);

        let svn = PathBuf::from_str("svn_from_cli").unwrap();
        let git = PathBuf::from_str("git_from_cli").unwrap();
        let config = select_or_create_config(
            Some(svn.clone()),
            Some(git.clone()),
            &mut history,
            &interactor,
        )
        .unwrap();

        assert_eq!(config.svn_dir, svn);
        assert_eq!(config.git_dir, git);
    }

    #[test]
    fn test_select_or_create_config_should_select_history_when_exists() {
        let mut storage = MockFileStorage::new();
        storage.expect_load().returning(|| {
            Ok(vec![HistoryRecord::new(
                1,
                PathBuf::from("svn_history"),
                PathBuf::from("git_history"),
            )])
        });
        storage.expect_save().returning(|_| Ok(()));
        let mut history = HistoryManager::new(storage).unwrap();

        let mut interactor = MockUserInteractor::new();
        interactor
            .expect_select_history_record()
            .returning(|_| Ok(0));
        interactor.expect_input_svn_dir().times(0);
        interactor.expect_input_git_dir().times(0);

        let config = select_or_create_config(None, None, &mut history, &interactor).unwrap();
        assert_eq!(config.svn_dir, PathBuf::from("svn_history"));
        assert_eq!(config.git_dir, PathBuf::from("git_history"));
    }
}
