//! 改进的Git提交功能测试
//!
//! 测试修复后的git_commit函数

mod common;

use common::*;
use std::process::Command;

/// 测试：改进后的git_commit应该能处理新文件
#[test]
fn test_improved_git_commit_can_handle_new_files() {
    let (_temp_dir, _svn_dir, git_dir) = create_test_dirs();

    // 初始化Git仓库
    Command::new("git")
        .arg("init")
        .current_dir(&git_dir)
        .output()
        .expect("初始化Git仓库失败");

    // 配置Git用户信息
    Command::new("git")
        .args(&["config", "user.name", "测试用户"])
        .current_dir(&git_dir)
        .output()
        .expect("配置Git用户名失败");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&git_dir)
        .output()
        .expect("配置Git邮箱失败");

    // 创建一个新的未跟踪文件
    let test_file = git_dir.join("new_file.txt");
    std::fs::write(&test_file, "新文件内容").expect("创建测试文件失败");

    // 检查Git状态，确认文件是未跟踪的
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(&git_dir)
        .output()
        .expect("获取Git状态失败");

    let status_output_str = String::from_utf8_lossy(&status_output.stdout);
    println!("Git状态输出:\n{}", status_output_str);

    // 验证文件是未跟踪的（以??开头）
    assert!(
        status_output_str.contains("?? new_file.txt"),
        "新文件应该是未跟踪状态，但实际状态是: {}",
        status_output_str
    );

    // 尝试使用改进后的git_commit_real函数提交
    let commit_result = svn2git::git_commit_real(&git_dir, "测试提交新文件");

    // 期望：改进后的实现应该成功
    assert!(
        commit_result.is_ok(),
        "改进后的git_commit应该能成功提交新文件"
    );

    if let Err(e) = commit_result {
        println!("提交失败，错误信息: {}", e);
        panic!("测试失败：改进后的git_commit仍然无法提交新文件");
    }

    // 再次检查Git状态，应该是干净的
    let status_after_commit_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(&git_dir)
        .output()
        .expect("获取提交后Git状态失败");

    let status_after_commit_str = String::from_utf8_lossy(&status_after_commit_output.stdout);
    println!("提交后Git状态:\n{}", status_after_commit_str);

    // 验证工作目录是干净的
    assert!(
        status_after_commit_str.trim().is_empty(),
        "提交后工作目录应该是干净的，但实际状态是: {}",
        status_after_commit_str
    );

    // 检查提交历史，应该有我们的提交
    let log_output = Command::new("git")
        .args(&["log", "--oneline", "-1"])
        .current_dir(&git_dir)
        .output()
        .expect("获取Git日志失败");

    let log_str = String::from_utf8_lossy(&log_output.stdout);
    println!("最新提交:\n{}", log_str);

    // 验证提交存在且包含我们的提交信息
    assert!(!log_str.trim().is_empty(), "应该有提交记录");
    assert!(
        log_str.contains("测试提交新文件") || log_str.contains("test commit"),
        "提交信息应该包含我们的消息"
    );

    println!("✅ 测试通过：改进后的git_commit能正确处理新文件");
}
