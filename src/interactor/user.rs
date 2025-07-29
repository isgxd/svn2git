use inquire::{Confirm, Select, Text};

use crate::{
    config::HistoryRecord,
    error::{Result, SyncError},
    ops::SvnLog,
};

/// 用户交互接口
#[cfg_attr(test, mockall::automock)]
pub trait UserInteractor {
    /// 选择历史记录
    ///
    /// # 参数
    ///
    /// * `records`: 历史记录
    ///
    /// # 返回
    ///
    /// 选择的记录索引
    fn select_history_record(&self, records: &[HistoryRecord]) -> Result<usize>;
    /// 输入 SVN 本地目录
    fn input_svn_dir(&self) -> Result<String>;
    /// 输入 Git 本地目录
    fn input_git_dir(&self) -> Result<String>;
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
    fn confirm_sync(&self, svn_logs: &[SvnLog]) -> bool;
}

/// 默认的用户交互器
pub struct DefaultUserInteractor;

impl UserInteractor for DefaultUserInteractor {
    fn select_history_record(&self, records: &[HistoryRecord]) -> Result<usize> {
        let options: Vec<String> = records.iter().map(|r| r.to_string()).collect();

        let selection = Select::new("选择一个历史记录", options)
            .with_starting_cursor(0)
            .prompt()?;

        records
            .iter()
            .position(|r| r.to_string().eq(&selection))
            .ok_or_else(|| SyncError::App("未找到所选记录".into()))
    }

    fn input_svn_dir(&self) -> Result<String> {
        Text::new("输入 SVN 文件夹：")
            .prompt()
            .map_err(|e| e.into())
    }

    fn input_git_dir(&self) -> Result<String> {
        Text::new("输入 Git 文件夹：")
            .prompt()
            .map_err(|e| e.into())
    }

    fn confirm_sync(&self, svn_logs: &[SvnLog]) -> bool {
        println!("SVN 更新日志：");
        for log in svn_logs {
            println!("{log:?}");
        }

        Confirm::new("是否执行同步？")
            .with_default(false)
            .prompt()
            .inspect_err(|e| eprintln!("询问是否同步时出现错误：{e}"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_history_record() {
        let records = vec![
            HistoryRecord::new(1, "svn_dir_1".into(), "git_dir_1".into()),
            HistoryRecord::new(2, "svn_dir_2".into(), "git_dir_2".into()),
        ];

        let interactor = DefaultUserInteractor;
        let selection = interactor.select_history_record(&records).unwrap();
        assert_eq!(selection, 1);
    }

    #[test]
    fn test_input_svn_dir() {
        let interactor = DefaultUserInteractor;
        let svn_dir = interactor.input_svn_dir().unwrap();
        assert_eq!(svn_dir, "svn".to_string());
    }

    #[test]
    fn test_input_git_dir() {
        let interactor = DefaultUserInteractor;
        let git_dir = interactor.input_git_dir().unwrap();
        assert_eq!(git_dir, "git".to_string());
    }

    #[test]
    fn test_confirm_sync() {
        let interactor = DefaultUserInteractor;
        let svn_logs: Vec<SvnLog> = vec![SvnLog {
            version: "1".into(),
            message: "message".into(),
        }];

        let result = interactor.confirm_sync(&svn_logs);
        assert!(result);
    }
}
