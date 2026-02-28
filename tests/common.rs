//! 测试公共模块
//!
//! 提供测试用的工具函数和mock数据

use std::path::PathBuf;
use tempfile::TempDir;

/// 创建测试用的临时目录
#[allow(dead_code)]
pub fn create_test_dirs() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let svn_dir = temp_dir.path().join("svn");
    let git_dir = temp_dir.path().join("git");

    std::fs::create_dir(&svn_dir).expect("创建SVN目录失败");
    std::fs::create_dir(&git_dir).expect("创建Git目录失败");

    (temp_dir, svn_dir, git_dir)
}

// 创建测试用的SVN日志
//
// 注意：这个函数会在我们修改SvnLog结构体后更新
// pub fn create_test_svn_log_simple() -> Vec<SvnLog> {
//     vec![
//         SvnLog {
//             version: "123".to_string(),
//             message: "修复了重要bug".to_string(),
//         },
//         SvnLog {
//             version: "124".to_string(),
//             message: "添加了新功能".to_string(),
//         },
//     ]
// }

/// 创建测试用的完整SVN日志（待实现）
///
/// 当我们扩展SvnLog结构体后，这个函数将返回包含author和date的完整日志
#[allow(dead_code)]
pub fn create_test_svn_log_full() -> String {
    MOCK_SVN_LOG_XML.to_string()
}

/// 模拟的SVN日志XML数据
#[allow(dead_code)]
pub const MOCK_SVN_LOG_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<log>
<logentry revision="123">
<author>张三</author>
<date>2024-07-27T10:30:00.000Z</date>
<msg>修复了重要bug</msg>
</logentry>
<logentry revision="124">
<author>李四</author>
<date>2024-07-27T14:20:00.000Z</date>
<msg>添加了新功能</msg>
</logentry>
</log>"#;
