# svn2git
SVN到Git仓库同步工具

[English Documentation](README.md)

一个命令行工具，用于将SVN仓库历史记录同步到本地Git仓库。

## 功能特性
- 将SVN版本同步为Git提交
- 同步前提供交互式确认
- 历史记录管理（列出/删除同步记录）
- 支持自定义SVN和Git目录路径

## 安装方法
1. 安装Rust工具链: https://www.rust-lang.org/tools/install
2. 从源码构建:
```bash
cargo install --path .
```

## 使用方法
```bash
svn2git [命令]
```

### 命令说明
- `sync`: 同步SVN到Git
  ```bash
  svn2git sync --svn-dir [SVN目录] --git-dir [Git目录]
  ```
  - `--svn-dir`: SVN工作副本路径（可选）
  - `--git-dir`: Git仓库路径（可选）

- `history`: 管理同步历史
  ```bash
  svn2git history list          # 列出所有同步记录
  svn2git history delete [ID]   # 按ID删除同步记录
  ```

## 使用示例
1. 初始化同步:
```bash
svn2git sync --svn-dir ./my-svn --git-dir ./my-git
```
2. 查看同步历史:
```bash
svn2git history list
```
3. 删除同步记录:
```bash
svn2git history delete 1
```

## 许可证
MIT协议
