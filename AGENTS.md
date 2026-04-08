# 项目开发记录（Rust 重写版）

## 项目要求
- 保持仓库整洁：不提交临时/构建产物，`.gitignore` 已涵盖 `target/` 等目录。
- 文档与脚本：核心文档放置于仓库根目录（本文件、README、LICENSE）；CI/CD workflow 位于 `.github/workflows/`。
- 许可证：GNU Affero General Public License v3.0 (`AGPL-3.0`)，Cargo.toml 与 README 已同步。
- 任务完成后和提交前请执行 `cargo fmt`（格式化）与必要检查；推送后务必查看 GitHub Actions 状态，确保 CI 全绿。

## 技术决策（Rust 版）
- 语言/版本：Rust stable，二进制名称 `aniorg`。
- 依赖：
  - 核心：`clap`（CLI）、`regex`（文件名解析）、`walkdir`（递归遍历）、`thiserror`（错误处理）、`tracing`（日志）
  - 元数据：`reqwest`（HTTP）、`serde`/`serde_json`（序列化）、`quick-xml`（NFO）、`tokio`（异步运行时）
  - 数据库：`rusqlite`（SQLite）
  - 压缩：`flate2`、`zip`
  - 云盘/远程：`tonic`/`prost`（gRPC）、`prost-types`、`bt_bencode`（种子解析）、`tower`、`tracing-subscriber`、`url`
  - 可选依赖（通过 features 启用）：
    - `tempfile`（测试）
    - `sha1`、`async-trait`
- 架构：
  - `parser` 负责文件名解析，产出 `AnimeFileInfo`。
  - `organizer` 负责文件移动/复制/硬链接及覆盖策略。
  - `error` 统一错误类型。
  - `metadata` 负责 Bangumi/TMDB 元数据获取与别名匹配。
  - `nfo` 负责生成 Kodi 兼容的 NFO 文件。
  - `scraper` 负责从 RSS/页面刮削动画数据（需 `scraper` feature）。
  - `rss` 负责 RSS 订阅管理与云盘同步（需 `clouddrive` feature）。
  - `main` 仅做 CLI 解析、遍历与调度。
- 注释规范：遵循 `docs.rs` 风格的文档注释，示例可 `cargo test` 运行。

## 功能特性（Feature Flags）

| Feature | 默认启用 | 说明 |
|---------|---------|------|
| `metadata` | ✅ | 元数据刮削（Bangumi/TMDB/NFO 生成） |
| `scraper` | ❌ | 刮削子命令：scrape/match/build-db/extract-aliases 等 |
| `llm-api` | ❌ | LLM API 集成（OpenCode/MiniMax） |
| `clouddrive` | ❌ | 云盘集成（115网盘/RSS 订阅管理） |

## 构建与发布
- CI：`.github/workflows/ci.yml` 覆盖 fmt、clippy、测试与文档构建，运行平台含 Ubuntu/Windows/macOS。
- Release：`.github/workflows/release.yml` 在推送 `v*` 标签时构建多平台二进制并上传 Release，同时尝试 `cargo publish`（需要 `CARGO_REGISTRY_TOKEN`）。
- Nightly：`.github/workflows/nightly.yml` 每日构建，启用所有 features。
- 别名更新：`.github/workflows/alias-update.yml` 自动处理别名请求 issue。
- 数据库构建：`.github/workflows/bangumi-db.yml` 从 Bangumi Archive 构建 SQLite 别名库。
- 安装方式：
  - 从 GitHub Releases 下载预编译二进制
  - 从源码构建：`cargo build --release`（默认启用 metadata feature）
  - 全功能构建：`cargo build --release --features "scraper clouddrive"`

## 使用要点
- 支持模式：`link`（默认，硬链接，需同一文件系统）、`move`、`copy`。
- 默认扩展名：`.mp4,.mkv,.avi,.mov,.wmv,.flv,.rmvb`，可通过 `--include-ext` 覆盖。
- 预览：`--dry-run`；详细日志：`--verbose`。
- 可选回退：仅在指定 `--fallback-on-link-failure=move|copy` 时，硬链接失败会按所选模式回退；未指定则失败即报错并跳过。
- 元数据刮削：启用 `--scrape-metadata` 后自动生成 NFO 文件并下载封面图片。

## CLI 子命令（需相应 feature）

### 刮削相关（`--features scraper`）
```bash
# 刮削近期更新
cargo run --features scraper -- scrape --days 7 --format json

# 匹配别名提案
cargo run --features scraper -- match --input scraped.json --format github

# 构建 SQLite 别名库
cargo run --features scraper -- build-db --output bangumi.db

# 从 dump 提取别名
cargo run --features scraper -- extract-aliases --download

# 合并新别名到数据库
cargo run --features scraper -- merge-aliases --input new_aliases.json

# 应用匹配的别名提案
cargo run --features scraper -- apply-matches --input proposals.json

# 创建别名请求 issue
cargo run --features scraper -- create-alias-issues --input uncertain.json --repo ModerRAS/anime-organizer
```

### RSS 订阅管理（`--features clouddrive`）
```bash
# 列出订阅
aniorg rss --list-subscriptions

# 添加订阅
aniorg rss --add-subscription --rss-url "https://example.com/rss" --rss-target "/anime" --rss-filter "720p"

# 单次执行
aniorg rss --clouddrive-url http://localhost:19798 --rss-url "https://example.com/rss" --rss-target "/anime"

# Daemon 模式
aniorg rss --daemon --clouddrive-url http://localhost:19798 --rss-interval 300
```

## 兼容性与注意事项
- 硬链接：跨设备会返回 `硬链接失败：源文件和目标必须在同一文件系统`；无自动回退，除非显式使用 `--fallback-on-link-failure`。
- 覆盖策略：目标文件存在时直接覆盖（先删除后写）。
- 行结束符：仓库以 LF 为主，Windows 提交会由 Git 自动转换 CRLF；如需统一可在本地配置 `core.autocrlf`。
- SQLite 别名库：首次使用 `--scrape-metadata` 时会自动下载/构建，也可手动通过 `build-db` 子命令构建。

## 未来改进（可选）
- 增加并行处理以提升大批量文件性能。
- 提供可配置的目标命名模板。
- 为发布流程添加签名和校验和产物。
