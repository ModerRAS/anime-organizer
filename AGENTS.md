# 项目开发记录（Rust 重写版）

## 项目要求
- 保持仓库整洁：不提交临时/构建产物，`.gitignore` 已涵盖 `target/` 等目录。
- 文档与脚本：核心文档放置于仓库根目录（本文件、README、LICENSE）；CI/CD workflow 位于 `.github/workflows/`。
- 许可证：GNU Affero General Public License v3.0 (`AGPL-3.0`)，Cargo.toml 与 README 已同步。

## 技术决策（Rust 版）
- 语言/版本：Rust stable，二进制名称 `aniorg`。
- 依赖：`clap`（CLI）、`regex`（文件名解析）、`walkdir`（递归遍历）、`thiserror`（错误处理）、`tempfile`（测试）。
- 架构：
  - `parser` 负责文件名解析，产出 `AnimeFileInfo`。
  - `organizer` 负责文件移动/复制/硬链接及覆盖策略。
  - `error` 统一错误类型。
  - `main` 仅做 CLI 解析、遍历与调度。
- 注释规范：遵循 `docs.rs` 风格的文档注释，示例可 `cargo test` 运行。

## 构建与发布
- CI：`.github/workflows/ci.yml` 覆盖 fmt、clippy、测试与文档构建，运行平台含 Ubuntu/Windows/macOS。
- Release：`.github/workflows/release.yml` 在推送 `v*` 标签时构建多平台二进制并上传 Release，同时尝试 `cargo publish`（需要 `CARGO_REGISTRY_TOKEN`）。
- 安装方式：
  - `cargo install anime-organizer`
  - 或从 GitHub Releases 下载预编译二进制。

## 使用要点
- 支持模式：`move`（默认）、`copy`、`link`（硬链接，需同一文件系统）。
- 默认扩展名：`.mp4,.mkv,.avi,.mov,.wmv,.flv,.rmvb`，可通过 `--include-ext` 覆盖。
- 预览：`--dry-run`；详细日志：`--verbose`。

## 兼容性与注意事项
- 硬链接：跨设备会返回 `硬链接失败：源文件和目标必须在同一文件系统`。
- 覆盖策略：目标文件存在时直接覆盖（先删除后写）。
- 行结束符：仓库以 LF 为主，Windows 提交会由 Git 自动转换 CRLF；如需统一可在本地配置 `core.autocrlf`。

## 未来改进（可选）
- 增加并行处理以提升大批量文件性能。
- 提供可配置的目标命名模板。
- 为发布流程添加签名和校验和产物。
