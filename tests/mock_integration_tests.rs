//! 使用Mock Git操作的集成测试
//!
//! 测试完整的SVN到Git同步流程，但不依赖真实的Git命令

mod common;

use common::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{cell::RefCell, path::Path};
use svn2git::{GitOperations, SyncError, git_commit_with_ops};

/// 简化的Mock Git操作实现，用于集成测试
struct TestMockGitOperations {
    repos: RefCell<HashMap<String, TestMockRepo>>, // path -> mock repo
}

#[derive(Debug, Clone)]
struct TestMockRepo {
    initialized: bool,
    files: Vec<String>,           // 记录未提交的文件
    commits: Vec<TestMockCommit>, // 记录提交历史
}

#[derive(Debug, Clone)]
struct TestMockCommit {
    message: String,
    #[allow(dead_code)]
    files: Vec<String>,
}

impl TestMockGitOperations {
    fn new() -> Self {
        Self {
            repos: RefCell::new(HashMap::new()),
        }
    }

    fn get_repo(&self, path: &Path) -> Option<TestMockRepo> {
        let path_str = path.to_string_lossy().to_string();
        self.repos.borrow().get(&path_str).cloned()
    }

    fn get_repo_mut(&self, path: &Path) -> std::cell::RefMut<'_, TestMockRepo> {
        let path_str = path.to_string_lossy().to_string();
        std::cell::RefMut::map(self.repos.borrow_mut(), |repos| {
            repos.entry(path_str).or_insert_with(|| TestMockRepo {
                initialized: false,
                files: Vec::new(),
                commits: Vec::new(),
            })
        })
    }

    /// 手动添加文件到Mock仓库状态中
    fn add_file_to_mock(&self, repo_path: &Path, file_path: &str) {
        let mut repo = self.get_repo_mut(repo_path);
        if !repo.files.contains(&file_path.to_string()) {
            repo.files.push(file_path.to_string());
        }
    }
}

impl GitOperations for TestMockGitOperations {
    fn init(&self, path: &Path) -> std::result::Result<(), SyncError> {
        println!("模拟Git初始化: {:?}", path);
        let path_str = path.to_string_lossy().to_string();
        let mut repos = self.repos.borrow_mut();
        if repos.contains_key(&path_str) {
            return Err(SyncError::App("Git仓库已经初始化".to_string()));
        }
        repos.insert(
            path_str,
            TestMockRepo {
                initialized: true,
                files: Vec::new(),
                commits: Vec::new(),
            },
        );
        Ok(())
    }

    fn config_user(
        &self,
        path: &Path,
        _name: &str,
        _email: &str,
    ) -> std::result::Result<(), SyncError> {
        println!("模拟Git用户配置: {:?}", path);
        Ok(())
    }

    fn add_all(&self, path: &Path) -> std::result::Result<(), SyncError> {
        println!("模拟添加所有文件到暂存区: {:?}", path);
        // add_all 不需要做任何实际操作，因为我们已经通过 add_file_to_mock 添加了文件
        Ok(())
    }

    fn commit(&self, path: &Path, message: &str) -> std::result::Result<(), SyncError> {
        println!("模拟提交: {} - {:?}", message, path);
        let mut repo = self.get_repo_mut(path);
        if !repo.initialized {
            return Err(SyncError::App("Git仓库未初始化".to_string()));
        }

        if repo.files.is_empty() {
            return Err(SyncError::App("没有暂存的文件可以提交".to_string()));
        }

        // 创建提交记录
        let commit = TestMockCommit {
            message: message.to_string(),
            files: repo.files.clone(),
        };

        repo.commits.push(commit);
        repo.files.clear(); // 清空未提交的文件

        Ok(())
    }

    fn status(&self, path: &Path) -> std::result::Result<String, SyncError> {
        println!("模拟Git状态查询: {:?}", path);
        if let Some(repo) = self.get_repo(path) {
            if !repo.initialized {
                return Err(SyncError::App("Git仓库未初始化".to_string()));
            }
            if repo.files.is_empty() {
                Ok(String::new()) // 无未跟踪文件，工作目录干净
            } else {
                Ok("?? some_untracked_file.txt\n".to_string()) // 有未跟踪文件
            }
        } else {
            Err(SyncError::App("Git仓库未初始化".to_string()))
        }
    }

    fn log(&self, path: &Path, count: Option<usize>) -> std::result::Result<String, SyncError> {
        println!("模拟Git日志查询: {:?}", path);
        if let Some(repo) = self.get_repo(path) {
            if !repo.initialized {
                return Err(SyncError::App("Git仓库未初始化".to_string()));
            }

            let limit = count.unwrap_or(repo.commits.len());
            let limited_commits: Vec<_> = repo.commits.iter().rev().take(limit).collect();

            let mut result = String::new();
            for commit in limited_commits {
                result.push_str(&format!("commit1 {}\n", commit.message));
            }

            Ok(result)
        } else {
            Err(SyncError::App("Git仓库未初始化".to_string()))
        }
    }

    fn is_clean(&self, path: &Path) -> std::result::Result<bool, SyncError> {
        println!("模拟检查工作目录是否干净: {:?}", path);
        if let Some(repo) = self.get_repo(path) {
            if !repo.initialized {
                return Err(SyncError::App("Git仓库未初始化".to_string()));
            }
            Ok(repo.files.is_empty())
        } else {
            Err(SyncError::App("Git仓库未初始化".to_string()))
        }
    }
}

/// 测试：Mock Git状态查询功能应该返回仓库信息
#[test]
fn test_mock_get_git_repository_info() {
    let (_temp_dir, _svn_dir, git_dir) = create_test_dirs();
    let git_ops = TestMockGitOperations::new();

    // 使用Mock Git初始化仓库
    git_ops.init(&git_dir).expect("初始化Git仓库失败");
    git_ops
        .config_user(&git_dir, "测试用户", "test@example.com")
        .expect("配置Git用户信息失败");

    // 验证Git仓库已初始化
    let status = git_ops.status(&git_dir).expect("获取Git状态失败");
    assert!(status.trim().is_empty(), "新仓库的状态应该是空的");

    println!("✅ Mock Git仓库已成功初始化在: {:?}", git_dir);
}

/// 测试：Mock Git提交功能应该正确处理所有更改
#[test]
fn test_mock_git_commit_includes_all_changes() {
    let (_temp_dir, _svn_dir, git_dir) = create_test_dirs();
    let git_ops = TestMockGitOperations::new();

    // 初始化Mock Git仓库
    git_ops.init(&git_dir).expect("初始化Git仓库失败");
    git_ops
        .config_user(&git_dir, "测试用户", "test@example.com")
        .expect("配置Git用户信息失败");

    // 创建测试文件（模拟文件创建）
    let test_file = git_dir.join("test.txt");
    std::fs::write(&test_file, "测试内容").expect("创建测试文件失败");

    // 通知Mock系统文件的存在
    git_ops.add_file_to_mock(&git_dir, "test.txt");

    // 检查Git状态，确认有未跟踪的文件
    let status_output = git_ops.status(&git_dir).expect("获取Git状态失败");
    println!("Mock Git状态输出:\n{}", status_output);

    // 使用重构后的git_commit_with_ops函数提交
    let commit_result = git_commit_with_ops(&git_ops, &git_dir, "测试提交");

    // 期望：现在的实现应该成功，因为我们使用了正确的Git操作抽象
    assert!(
        commit_result.is_ok(),
        "重构后的git_commit应该能成功提交更改"
    );

    if let Err(e) = commit_result {
        println!("提交失败，错误信息: {}", e);
        panic!("测试失败：重构后的git_commit仍然无法提交更改");
    }

    // 再次检查Git状态，应该是干净的
    let status_after_commit = git_ops.status(&git_dir).expect("获取提交后Git状态失败");
    println!("提交后Mock Git状态:\n{}", status_after_commit);

    // 验证工作目录是干净的
    assert!(
        status_after_commit.trim().is_empty(),
        "提交后工作目录应该是干净的，但实际状态是: {}",
        status_after_commit
    );

    // 检查提交历史，应该有我们的提交
    let log_output = git_ops.log(&git_dir, Some(1)).expect("获取Git日志失败");
    println!("最新提交:\n{}", log_output);

    // 验证提交存在且包含我们的提交信息
    assert!(!log_output.trim().is_empty(), "应该有提交记录");
    assert!(
        log_output.contains("测试提交"),
        "提交信息应该包含我们的消息"
    );

    println!("✅ 测试通过：重构后的git_commit能正确处理所有更改");
}

/// 测试：Mock Git命令失败时应该提供详细的错误信息
#[test]
fn test_mock_git_command_error_details() {
    let git_ops = TestMockGitOperations::new();
    let invalid_path = PathBuf::from("/不存在的路径");

    // 在未初始化的路径上获取状态应该失败
    let result = git_ops.status(&invalid_path);

    assert!(result.is_err(), "应该返回错误");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // 验证错误信息包含有用的调试信息
    assert!(!error_msg.is_empty(), "错误信息不应该为空");
    println!("Mock Git错误信息: {}", error_msg);

    // 期望错误信息指明是仓库未初始化
    assert!(
        error_msg.contains("未初始化") || error_msg.contains("init"),
        "错误信息应该指明仓库未初始化"
    );

    println!("✅ 测试通过：Mock Git错误信息足够详细");
}

/// 测试：Mock Git应该正确处理多次提交
#[test]
fn test_mock_git_multiple_commits_flow() {
    let (_temp_dir, _svn_dir, git_dir) = create_test_dirs();
    let git_ops = TestMockGitOperations::new();

    // 初始化Mock Git仓库
    git_ops.init(&git_dir).expect("初始化Git仓库失败");
    git_ops
        .config_user(&git_dir, "测试用户", "test@example.com")
        .expect("配置Git用户信息失败");

    // 模拟多次提交
    let commits = [
        ("初始提交", "README.md"),
        ("添加源代码", "src/main.rs"),
        ("修复bug", "src/bugfix.rs"),
    ];

    for (message, file) in &commits {
        // 模拟文件创建
        let file_path = git_dir.join(file);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("创建父目录失败");
        }
        std::fs::write(&file_path, format!("内容: {}", file)).expect("创建测试文件失败");

        // 通知Mock系统文件的存在
        git_ops.add_file_to_mock(&git_dir, file);

        // 使用重构后的git_commit_with_ops函数提交
        git_commit_with_ops(&git_ops, &git_dir, message)
            .unwrap_or_else(|_| panic!("提交失败: {}", message));

        println!("✅ 提交成功: {}", message);
    }

    // 验证所有提交都存在
    let all_logs = git_ops.log(&git_dir, None).expect("获取提交历史失败");
    let log_lines: Vec<&str> = all_logs.lines().collect();

    assert_eq!(log_lines.len(), 3, "应该有3个提交记录");

    // 验证每个提交的信息
    for (expected_message, _) in commits.iter().rev() {
        assert!(
            all_logs.contains(expected_message),
            "日志应该包含提交: {}",
            expected_message
        );
    }

    // 验证工作目录是干净的
    let is_clean = git_ops.is_clean(&git_dir).expect("检查工作目录状态失败");
    assert!(is_clean, "所有提交后工作目录应该是干净的");

    println!("✅ 测试通过：Mock Git正确处理多次提交流程");
}

/// 测试：Mock Git应该正确处理SVN同步场景
#[test]
fn test_mock_git_svn_sync_scenario() {
    let (_temp_dir, _svn_dir, git_dir) = create_test_dirs();
    let git_ops = TestMockGitOperations::new();

    // 初始化Git仓库
    git_ops.init(&git_dir).expect("初始化Git仓库失败");
    git_ops
        .config_user(&git_dir, "SVN同步用户", "sync@example.com")
        .expect("配置Git用户信息失败");

    // 模拟SVN同步场景：从SVN日志创建Git提交
    let svn_logs = vec![
        ("123", "修复重要bug"),
        ("124", "添加新功能"),
        ("125", "更新文档"),
    ];

    for (version, message) in &svn_logs {
        // 模拟SVN更新后的文件变化
        let file_path = git_dir.join(format!("svn_r{}.txt", version));
        std::fs::write(&file_path, format!("SVN版本{}的内容", version))
            .expect("创建SVN版本文件失败");

        // 通知Mock系统文件的存在
        git_ops.add_file_to_mock(&git_dir, &format!("svn_r{}.txt", version));

        // 使用与sync.rs相同的提交格式
        let commit_message = format!("SVN: {}", message);
        git_commit_with_ops(&git_ops, &git_dir, &commit_message)
            .unwrap_or_else(|_| panic!("SVN同步提交失败: r{}", version));

        println!("✅ SVN同步提交成功: r{} - {}", version, message);
    }

    // 验证同步结果
    let final_logs = git_ops.log(&git_dir, None).expect("获取最终提交历史失败");

    for (version, message) in &svn_logs {
        let expected_commit_message = format!("SVN: {}", message);
        assert!(
            final_logs.contains(&expected_commit_message),
            "同步历史应该包含: r{} - {}",
            version,
            message
        );
    }

    println!("✅ 测试通过：Mock Git正确处理SVN同步场景");
}
