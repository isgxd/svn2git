use std::{path::PathBuf, process::Command};

use roxmltree::Document;

use crate::error::{Result, SyncError};

/// SVN 日志
#[derive(Debug, Clone)]
pub struct SvnLog {
    pub version: String,
    pub message: String,
}

/// 获取 SVN 日志
///
/// # 参数
///
/// * `path`: SVN 本地目录
/// * `git_log`: Git 日志信息，可选
///
/// # 返回
///
/// SVN 日志列表
pub fn get_svn_logs(path: &PathBuf) -> Result<Vec<SvnLog>> {
    println!("正在获取 SVN 日志");

    let mut cmd = Command::new("svn");
    cmd.arg("log")
        .arg("--xml")
        .arg("-r")
        .arg("BASE:HEAD")
        .arg(path);

    let output = cmd.output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(SyncError::App(format!(
            "svn log 命令执行失败，错误信息：{err}"
        )));
    }

    parse_svn_log_xml(&output.stdout)
}

/// 解析 SVN 日志 XML
fn parse_svn_log_xml(xml: &[u8]) -> Result<Vec<SvnLog>> {
    let xml_str = str::from_utf8(xml)?;
    let doc = Document::parse(xml_str)?;

    let root = doc.root_element();
    if root.tag_name().name() != "log" {
        return Err(SyncError::App("无效的 XML 根，预期是 <log>".into()));
    }

    let mut logs = Vec::new();
    for entry in root
        .children()
        .filter(|e| e.is_element() && e.tag_name().name() == "logentry")
    {
        let version = entry
            .attribute("revision")
            .ok_or(SyncError::App("日志条目中缺少 revision 属性".into()))?
            .to_string();

        let message = get_svn_msg(entry);
        if message.is_empty() {
            // 允许空消息，但记录警告
            // 某些SVN提交可能确实为空消息，这是合法的
            println!("警告: SVN版本 {} 的提交消息为空", version);
        }

        logs.push(SvnLog { version, message });
    }

    Ok(logs)
}

/// 获取 SVN 日志消息
///
/// # 参数
///
/// * `entry`: SVN 日志条目
fn get_svn_msg(entry: roxmltree::Node<'_, '_>) -> String {
    let mut message = String::new();
    for child in entry.children().filter(|e| e.is_element()) {
        if child.tag_name().name() == "msg" {
            message = child.text().unwrap_or_default().trim().to_string();
            break;
        }
    }
    message
}

/// 拉取 SVN 指定版本到本地
///
/// # 参数
///
/// * `path`: SVN 本地目录
/// * `rev`: SVN 版本
pub fn svn_update_to_rev(path: &PathBuf, rev: &str) -> Result<()> {
    println!("正在拉取 SVN 版本 {rev} 到本地");

    let output = Command::new("svn")
        .arg("update")
        .arg("-r")
        .arg(rev)
        .current_dir(path)
        .output()?;
    if !output.status.success() {
        return Err(SyncError::App(format!(
            "svn 更新到 {rev} 失败，错误信息：{output:?}"
        )));
    }

    println!("SVN 更新到 {rev} 成功");
    Ok(())
}
