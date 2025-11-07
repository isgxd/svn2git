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

        match Confirm::new("是否执行同步？").with_default(false).prompt() {
            Ok(confirm) => confirm,
            Err(e) => {
                eprintln!("询问是否同步时出现错误：{e}");
                eprintln!("由于交互错误，将取消同步操作以确保安全");
                false // 安全默认值：出错时取消同步，避免意外操作
            }
        }
    }
}

/// 测试用Mock用户交互器，用于测试
#[cfg(test)]
pub struct TestUserInteractor {
    /// 预设的选择记录索引
    pub selected_index: usize,
    /// 预设的SVN目录输入
    pub svn_dir_input: String,
    /// 预设的Git目录输入
    pub git_dir_input: String,
    /// 预设的同步确认结果
    pub confirm_result: bool,
}

#[cfg(test)]
impl Default for TestUserInteractor {
    fn default() -> Self {
        Self {
            selected_index: 0,
            svn_dir_input: "svn".to_string(),
            git_dir_input: "git".to_string(),
            confirm_result: true,
        }
    }
}

#[cfg(test)]
impl TestUserInteractor {
    /// 创建新的Mock交互器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置选择记录的索引
    pub fn with_selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    /// 设置SVN目录输入
    pub fn with_svn_dir(mut self, dir: &str) -> Self {
        self.svn_dir_input = dir.to_string();
        self
    }

    /// 设置Git目录输入
    pub fn with_git_dir(mut self, dir: &str) -> Self {
        self.git_dir_input = dir.to_string();
        self
    }

    /// 设置同步确认结果
    pub fn with_confirm_result(mut self, result: bool) -> Self {
        self.confirm_result = result;
        self
    }
}

#[cfg(test)]
impl UserInteractor for TestUserInteractor {
    fn select_history_record(&self, records: &[HistoryRecord]) -> Result<usize> {
        if records.is_empty() {
            return Err(SyncError::App("没有历史记录可选择".into()));
        }
        if self.selected_index >= records.len() {
            return Err(SyncError::App("选择索引超出范围".into()));
        }
        Ok(self.selected_index)
    }

    fn input_svn_dir(&self) -> Result<String> {
        Ok(self.svn_dir_input.clone())
    }

    fn input_git_dir(&self) -> Result<String> {
        Ok(self.git_dir_input.clone())
    }

    fn confirm_sync(&self, _svn_logs: &[SvnLog]) -> bool {
        self.confirm_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：TestUserInteractor应该能正确选择历史记录
    #[test]
    fn test_test_user_interactor_select_history_record() {
        let records = vec![
            HistoryRecord::new(1, "svn_dir_1".into(), "git_dir_1".into()),
            HistoryRecord::new(2, "svn_dir_2".into(), "git_dir_2".into()),
        ];

        let interactor = TestUserInteractor::new().with_selected_index(1);
        let selection = interactor.select_history_record(&records).unwrap();
        assert_eq!(selection, 1);
    }

    /// 测试：TestUserInteractor应该在记录为空时返回错误
    #[test]
    fn test_test_user_interactor_select_history_record_empty() {
        let records: Vec<HistoryRecord> = vec![];
        let interactor = TestUserInteractor::new();

        let result = interactor.select_history_record(&records);
        assert!(result.is_err());
    }

    /// 测试：TestUserInteractor应该能正确输入SVN目录
    #[test]
    fn test_test_user_interactor_input_svn_dir() {
        let interactor = TestUserInteractor::new().with_svn_dir("test_svn");
        let svn_dir = interactor.input_svn_dir().unwrap();
        assert_eq!(svn_dir, "test_svn");
    }

    /// 测试：TestUserInteractor应该能正确输入Git目录
    #[test]
    fn test_test_user_interactor_input_git_dir() {
        let interactor = TestUserInteractor::new().with_git_dir("test_git");
        let git_dir = interactor.input_git_dir().unwrap();
        assert_eq!(git_dir, "test_git");
    }

    /// 测试：TestUserInteractor应该能正确确认同步
    #[test]
    fn test_test_user_interactor_confirm_sync() {
        let interactor = TestUserInteractor::new().with_confirm_result(false);
        let svn_logs: Vec<SvnLog> = vec![SvnLog {
            version: "1".into(),
            message: "message".into(),
        }];

        let result = interactor.confirm_sync(&svn_logs);
        assert!(!result);
    }
}
