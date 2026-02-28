//! 改进的Git提交功能测试
//!
//! 使用Mock验证提交逻辑，不依赖真实git命令

mod common;

use common::*;
use svn2git::{GitOperations, MockGitOperations, git_commit_with_ops};

/// 测试：提交逻辑应该能处理新文件（纯Mock）
#[test]
fn test_improved_git_commit_can_handle_new_files_without_real_git() {
    let (_temp_dir, _svn_dir, git_dir) = create_test_dirs();
    let git_ops = MockGitOperations::new();

    git_ops.init(&git_dir).expect("初始化Mock Git仓库失败");
    git_ops
        .config_user(&git_dir, "测试用户", "test@example.com")
        .expect("配置Mock Git用户失败");
    git_ops
        .add_file_to_mock(&git_dir, "new_file.txt")
        .expect("添加Mock文件失败");

    let status_output = git_ops.status(&git_dir).expect("获取Mock Git状态失败");
    assert!(!status_output.trim().is_empty(), "提交前应有待处理变更");

    let commit_result = git_commit_with_ops(&git_ops, &git_dir, "测试提交新文件");

    assert!(
        commit_result.is_ok(),
        "改进后的git_commit应该能成功提交新文件"
    );

    if let Err(e) = commit_result {
        println!("提交失败，错误信息: {}", e);
        panic!("测试失败：改进后的git_commit仍然无法提交新文件");
    }

    let status_after_commit_str = git_ops
        .status(&git_dir)
        .expect("获取提交后Mock Git状态失败");
    assert!(
        status_after_commit_str.trim().is_empty(),
        "提交后工作目录应该是干净的，但实际状态是: {}",
        status_after_commit_str
    );

    let log_str = git_ops
        .log(&git_dir, Some(1))
        .expect("获取Mock Git日志失败");
    assert!(!log_str.trim().is_empty(), "应该有提交记录");
    assert!(
        log_str.contains("测试提交新文件"),
        "提交信息应该包含我们的消息"
    );
}
