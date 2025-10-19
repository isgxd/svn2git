//! 集成测试模块
//!
//! 测试完整的SVN到Git同步流程，使用统一的依赖注入架构

mod common;

use common::*;
use svn2git::{
    GitOperations, MockGitOperations, ProviderType, git_commit_with_ops, test_utils::TestFactory,
};

/// 测试：Mock Git状态查询功能应该返回仓库信息
#[test]
fn test_mock_git_repository_info() {
    let git_ops = MockGitOperations::new();
    let config = TestFactory::create_sync_config(false); // 使用Mock

    // 使用Mock Git初始化仓库
    git_ops.init(&config.git_dir).expect("初始化Git仓库失败");
    git_ops
        .config_user(&config.git_dir, "测试用户", "test@example.com")
        .expect("配置Git用户信息失败");

    // 验证Git仓库已初始化
    let status = git_ops.status(&config.git_dir).expect("获取Git状态失败");
    assert!(status.trim().is_empty(), "新仓库的状态应该是空的");

    println!("✅ Mock Git仓库已成功初始化在: {:?}", config.git_dir);
}

/// 测试：Mock Git提交功能应该正确处理所有更改
#[test]
fn test_mock_git_commit_includes_all_changes() {
    let config = TestFactory::create_sync_config(false); // 使用Mock
    let git_ops = MockGitOperations::new();

    // 初始化Mock Git仓库
    git_ops.init(&config.git_dir).expect("初始化Git仓库失败");
    git_ops
        .config_user(&config.git_dir, "测试用户", "test@example.com")
        .expect("配置Git用户信息失败");

    // 添加测试文件到Mock系统
    git_ops
        .add_file_to_mock(&config.git_dir, "test.txt")
        .expect("添加文件失败");
    git_ops
        .add_file_to_mock(&config.git_dir, "src/main.rs")
        .expect("添加文件失败");

    // 检查Git状态，确认有未跟踪的文件
    let status_output = git_ops.status(&config.git_dir).expect("获取Git状态失败");
    println!("Mock Git状态输出:\n{}", status_output);

    // 使用重构后的git_commit_with_ops函数提交
    let commit_result = git_commit_with_ops(&git_ops, &config.git_dir, "测试提交");

    // 期望：使用Mock实现的提交应该成功
    assert!(commit_result.is_ok(), "Mock Git提交应该能成功提交更改");

    if let Err(e) = commit_result {
        println!("提交失败，错误信息: {}", e);
        panic!("测试失败：Mock Git提交无法提交更改");
    }

    // 再次检查Git状态，应该是干净的
    let status_after_commit = git_ops
        .status(&config.git_dir)
        .expect("获取提交后Git状态失败");
    println!("提交后Mock Git状态:\n{}", status_after_commit);

    // 验证工作目录是干净的
    assert!(
        status_after_commit.trim().is_empty(),
        "提交后工作目录应该是干净的，但实际状态是: {}",
        status_after_commit
    );

    // 检查提交历史，应该有我们的提交
    let log_output = git_ops
        .log(&config.git_dir, Some(1))
        .expect("获取Git日志失败");
    println!("最新提交:\n{}", log_output);

    // 验证提交存在且包含我们的提交信息
    assert!(!log_output.trim().is_empty(), "应该有提交记录");
    assert!(
        log_output.contains("测试提交"),
        "提交信息应该包含我们的消息"
    );

    println!("✅ 测试通过：Mock Git正确处理所有更改");
}

/// 测试：Mock Git应该正确处理多次提交
#[test]
fn test_mock_git_multiple_commits_flow() {
    let config = TestFactory::create_sync_config(false); // 使用Mock
    let git_ops = MockGitOperations::new();

    // 初始化Mock Git仓库
    git_ops.init(&config.git_dir).expect("初始化Git仓库失败");
    git_ops
        .config_user(&config.git_dir, "测试用户", "test@example.com")
        .expect("配置Git用户信息失败");

    // 模拟多次提交
    let commits = vec![("1", "初始提交"), ("2", "添加功能"), ("3", "修复bug")];

    for (version, message) in commits.iter() {
        let file_path = format!("svn_r{}.txt", version);

        // 模拟文件创建
        git_ops
            .add_file_to_mock(&config.git_dir, &file_path)
            .expect("添加文件失败");

        // 使用与sync.rs相同的提交格式
        let commit_message = format!("SVN: {}", message);
        git_commit_with_ops(&git_ops, &config.git_dir, &commit_message)
            .expect(&format!("SVN同步提交失败: r{}", version));

        println!("✅ 提交成功: r{} - {}", version, message);
    }

    // 验证所有提交都存在
    let all_logs = git_ops
        .log(&config.git_dir, None)
        .expect("获取提交历史失败");
    let log_lines: Vec<&str> = all_logs.lines().collect();

    assert_eq!(log_lines.len(), 3, "应该有3个提交记录");

    // 验证每个提交的信息
    for (expected_version, expected_message) in commits.iter() {
        let expected_commit_message = format!("SVN: {}", expected_message);
        assert!(
            all_logs.contains(&expected_commit_message),
            "同步历史应该包含: r{} - {}",
            expected_version,
            expected_message
        );
    }

    // 验证工作目录是干净的
    let is_clean = git_ops
        .is_clean(&config.git_dir)
        .expect("检查工作目录状态失败");
    assert!(is_clean, "所有提交后工作目录应该是干净的");

    println!("✅ 测试通过：Mock Git正确处理多次提交流程");
}

/// 测试：使用TestFactory创建SyncConfig应该正确工作
#[test]
fn test_sync_config_with_test_factory() {
    // 使用TestFactory创建Mock SyncConfig
    let config = TestFactory::create_sync_config(false);
    assert_eq!(config.git_provider, ProviderType::Mock);

    // 使用TestFactory创建Real SyncConfig
    let config_real = TestFactory::create_sync_config(true);
    assert_eq!(config_real.git_provider, ProviderType::Real);

    println!("✅ SyncConfig使用TestFactory创建成功");
}

/// 测试：SVN日志XML数据格式验证
#[test]
fn test_svn_log_xml_format() {
    let mock_xml = MOCK_SVN_LOG_XML.as_bytes();

    // 验证XML格式正确
    let xml_content = String::from_utf8_lossy(mock_xml);
    assert!(xml_content.contains("<log>"));
    assert!(xml_content.contains("</log>"));
    assert!(xml_content.contains("<logentry"));
    assert!(xml_content.contains("<author>"));
    assert!(xml_content.contains("<date>"));
    assert!(xml_content.contains("<msg>"));

    // 验证包含预期的数据
    assert!(xml_content.contains("张三"));
    assert!(xml_content.contains("李四"));
    assert!(xml_content.contains("修复了重要bug"));
    assert!(xml_content.contains("添加了新功能"));

    println!("✅ SVN日志XML格式验证通过");
}
