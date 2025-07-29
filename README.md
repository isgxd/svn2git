# svn2git
将服务端的SVN记录同步到本地Git仓库

[中文版文档](README_CN.md)

A command-line tool for synchronizing SVN repository history to a local Git repository.

## Features
- Synchronize SVN revisions to Git commits
- Interactive confirmation before synchronization
- History management (list/delete sync records)
- Support for custom SVN and Git directory paths

## Installation
1. Install Rust toolchain: https://www.rust-lang.org/tools/install
2. Build from source:
```bash
cargo install --path .
```

## Usage
```bash
svn2git [COMMAND]
```

### Commands
- `sync`: Synchronize SVN to Git
  ```bash
  svn2git sync --svn-dir [SVN_DIR] --git-dir [GIT_DIR]
  ```
  - `--svn-dir`: Path to SVN working copy (optional)
  - `--git-dir`: Path to Git repository (optional)

- `history`: Manage sync history
  ```bash
  svn2git history list          # List all sync records
  svn2git history delete [ID]   # Delete a sync record by ID
  ```

## Example
1. Initialize sync:
```bash
svn2git sync --svn-dir ./my-svn --git-dir ./my-git
```
2. Check sync history:
```bash
svn2git history list
```
3. Delete a sync record:
```bash
svn2git history delete 1
```

## License
MIT
