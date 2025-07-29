use std::{path::PathBuf, process::Command};

use crate::error::{Result, SyncError};

/// 提交 Git 更改
///
/// # 参数
///
/// * `path`: Git 本地目录
/// * `message`: 提交消息
pub fn git_commit(path: &PathBuf, message: &str) -> Result<()> {
    println!("正在提交 Git 更改");

    let output = Command::new("git")
        .arg("commit")
        .arg("-am")
        .arg(message)
        .current_dir(path)
        .output()?;
    if !output.status.success() {
        return Err(SyncError::App(format!(
            "git commit 命令执行失败，错误信息：{output:?}"
        )));
    }

    println!("Git 提交成功：{message}");
    Ok(())
}
