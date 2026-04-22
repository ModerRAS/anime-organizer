# anime-organizer Copilot 指南

开始处理任务前，先阅读仓库根目录的 `AGENTS.md`，那里记录了当前项目的模块分布、验证命令和常见约束。

## 仓库概览
- 这是一个 Rust CLI 工具，用于批量整理动漫视频文件，并提供元数据刮削、RSS/CloudDrive 同步、Torrent 标题爬取等可选能力。
- 代码入口主要在 `src\main.rs`，核心模块包括 `src\parser.rs`、`src\organizer.rs`、`src\metadata\`、`src\rss\`、`src\scraper\` 和 `src\torrent\`。
- 项目强调 **Pure Rust**；不要引入 Python 或其他语言实现同类功能。

## 默认 PR 工作流
- 除非用户明确要求复用现有分支或继续已有 PR，否则先同步最新 `origin/master`，再从更新后的 `master` 新建分支。
- 实现需求时保持改动聚焦，只修改与当前任务直接相关的代码、文档和测试。
- 完成后默认推送分支并创建或更新 PR。
- 必须检查 PR 的 CI 状态；如果有失败，读取失败 job 日志，修复问题后继续推送更新。
- 必须检查 PR 对话、review comments 和相关讨论；如果反馈要求代码改动，应直接修改并更新 PR，而不只是文字回复。
- 如果因权限不足、外部服务异常或需求不明确而受阻，需要明确说明阻塞原因。

## 验证命令
仓库当前要求的验证命令如下：

```bash
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test
cargo doc --no-deps --document-private-items
```

如果改动只涉及部分功能，也应至少运行与改动范围相关的现有命令；不要引入新的 lint/test 工具。

## 开发提示
- CLI 参数和子命令定义在 `src\main.rs`。
- 文件名解析规则集中在 `src\parser.rs`。
- 文件操作逻辑集中在 `src\organizer.rs`。
- 元数据相关逻辑集中在 `src\metadata\` 和 `src\nfo.rs`。
- RSS / CloudDrive 逻辑集中在 `src\rss\`，需要 `clouddrive` feature。
- Torrent 标题爬取逻辑集中在 `src\torrent\`，需要 `torrent-scraper` feature。

## 项目约束
- 需要保持 feature-gated 模块风格：使用 `#[cfg(feature = \"...\")]` 控制可选功能。
- 默认临时文件放在 `tmp\`。
- 对跨文件系统硬链接失败的场景，不要绕过现有 `CrossDeviceLink` / fallback 逻辑。
- 修改文档时，确保 README 和相关知识文档与实际 CLI 行为保持一致。
