# anime-organizer

[![CI](https://github.com/ModerRAS/anime-organizer/actions/workflows/ci.yml/badge.svg)](https://github.com/ModerRAS/anime-organizer/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/anime-organizer.svg)](https://crates.io/crates/anime-organizer)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

轻量级、跨平台命令行工具，专为动漫收藏者批量整理视频文件，支持硬链接模式实现零额外空间占用。

[English](#english) | [中文](#中文)

---

## 中文

### 📖 项目描述

anime-organizer（简称 aniorg）是一款专为动漫收藏者设计的命令行工具，旨在自动化整理下载目录中的动漫视频文件。

**核心功能：**

- 自动解析文件名，识别动漫名称、集数、发布组等信息
- 支持硬链接模式，整理后不占用额外磁盘空间
- 自动生成 Kodi 兼容的 NFO 元数据文件
- 下载封面海报和背景图
- 支持 RSS 订阅和 115 网盘同步（需启用 clouddrive feature）

**适用场景：**

- 整理下载目录中的动漫文件
- 自动化管理动漫媒体库
- 为 Kodi/Jellyfin/Plex 等媒体中心生成元数据

### 🚀 功能特性

- **智能解析**: 自动识别 `[发布组] 动漫名 - 集数 [标签].ext` 格式
- **灵活整理**: 重构为 `动漫名/Season N/集数 [标签].ext` 结构
- **多种模式**: 支持移动、复制、硬链接三种操作模式
- **元数据刮削**: 基于 Bangumi 生成 Kodi 兼容的 NFO 文件
- **封面下载**: 通过 TMDB 下载海报和背景图，失败时回退 AniDB 海报
- **别名匹配**: 支持本地 `bangumi.db` 别名库和自定义 JSON 覆盖，缺失时回退到 Bangumi 名称/搜索匹配
- **RSS 订阅**: 支持 RSS 自动订阅和 115 网盘同步（需 `clouddrive` feature）
- **跨平台**: 支持 Windows、Linux、macOS
- **零依赖运行**: 单文件部署，无需外部配置
- **高性能**: Rust 原生实现，极速处理

### 📥 安装方式

#### 方式一：下载预编译二进制

从 [GitHub Releases](https://github.com/ModerRAS/anime-organizer/releases) 下载对应平台的二进制文件：

| 平台 | 文件 |
|------|------|
| Windows x64 | `aniorg-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `aniorg-aarch64-pc-windows-msvc.zip` |
| Linux x64 | `aniorg-x86_64-unknown-linux-gnu.tar.gz` |
| Linux x64 (musl) | `aniorg-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 | `aniorg-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x64 | `aniorg-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 | `aniorg-aarch64-apple-darwin.tar.gz` |

#### 方式二：从源码构建

```bash
git clone https://github.com/ModerRAS/anime-organizer.git
cd anime-organizer

# 默认构建（启用 metadata feature）
cargo build --release

# 全功能构建
cargo build --release --features "scraper clouddrive torrent-scraper"
```

编译后的二进制文件位于 `target/release/aniorg`。

### 🎯 快速开始

#### 基本用法

```bash
# 硬链接模式（默认，零额外空间）
aniorg --source="/path/to/downloads"

# 复制模式
aniorg --source="/path/to/downloads" --mode=copy --target="/path/to/anime"

# 启用硬链接失败回退为复制
aniorg --source="/path/to/downloads" --fallback-on-link-failure=copy

# 启用分季目录结构
aniorg --source="/path/to/downloads" --season-mode --target="/path/to/anime"

# 生成 NFO 和海报
aniorg --source="/path/to/downloads" --scrape-metadata --tmdb-api-key="YOUR_TMDB_KEY"
```

#### 预览模式

在实际操作前先预览变更：

```bash
aniorg --source="/path/to/downloads" --dry-run --verbose
```

### 📋 参数说明

| 参数 | 缩写 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|------|--------|------|
| `--source` | `-s` | string | ✅ | - | 源目录路径 |
| `--target` | `-t` | string | ❌ | source | 目标根目录 |
| `--mode` | `-m` | enum | ❌ | link | 操作模式：move/copy/link |
| `--dry-run` | | bool | ❌ | false | 仅预览不执行 |
| `--include-ext` | | string | ❌ | mp4,mkv,... | 处理的扩展名（逗号分隔） |
| `--verbose` | `-v` | bool | ❌ | false | 显示详细日志 |
| `--fallback-on-link-failure` | | enum | ❌ | - | 硬链接失败时回退模式：move 或 copy（默认不回退） |
| `--season-mode` / `--分季` | | bool | ❌ | false | 按 `番名/Season N/` 结构整理多季作品 |
| `--scrape-metadata` / `--刮削` | | bool | ❌ | false | 启用 Bangumi/TMDB 元数据刮削 |
| `--tmdb-api-key` | | string | ❌ | - | TMDB API Key，用于下载海报和背景图 |
| `--alias-file` | | string | ❌ | - | 自定义别名 JSON 文件，覆盖本地 `bangumi.db` 中的同名项 |
| `--no-images` | | bool | ❌ | false | 只生成 NFO，不下载图片 |
| `--force-overwrite` | | bool | ❌ | false | 覆盖已有的 NFO 和图片文件 |
| `--bangumi-cache` | | string | ❌ | 系统临时目录 | Bangumi 缓存目录 |
| `--metadata-source` | | string | ❌ | - | 指定本地 `subject.jsonlines` 或其所在目录 |
| `--help` | `-h` | bool | ❌ | false | 显示帮助 |
| `--version` | `-V` | bool | ❌ | false | 显示版本 |

### 🧾 元数据刮削

启用 `--scrape-metadata` 后，程序会：

- 优先使用 `--alias-file` JSON 和本地 `bangumi.db` 别名库匹配 Bangumi 条目，缺失时回退到 Bangumi 名称/搜索匹配
- 在动画根目录生成 `tvshow.nfo`
- 在 `Season N/` 目录下生成与视频同名的 `*.nfo`
- 如果提供了 TMDB API Key，则下载 `poster.jpg`、`fanart.jpg` 和 `seasonXX-poster.jpg`

`--alias-file` 可直接复用 `extract-aliases` 导出的 JSON 对象格式，例如：

```json
{
  "芙莉莲": {
    "bangumi_id": 444444,
    "name": "葬送のフリーレン",
    "tmdb_id": 209867,
    "anidb_id": 18597
  }
}
```

### 🎨 文件命名格式

#### 支持的源文件名格式

```
[发布组] 动漫名称（可含季度） - 集数 [标签信息].扩展名
```

示例：
- `[ANi] 妖怪旅館營業中 貳 - 07 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4`
- `[SubsPlease] 间谍过家家 - 12 [1080p].mkv`
- `[EMBER] 进击的巨人 The Final Season - 01 [1080p][Multiple Subtitle].avi`

#### 目标文件结构

```
动漫名称（含季度）/
├── Season 1/
│   ├── 01 [标签信息].扩展名
│   └── 02 [标签信息].扩展名
├── Season 2/
│   ├── 01 [标签信息].扩展名
│   └── ...
├── tvshow.nfo
├── poster.jpg
└── fanart.jpg
```

### 🔗 硬链接说明

硬链接是推荐的整理方式，具有以下优势：

- **零额外空间**: 不占用额外磁盘空间
- **快速操作**: 几乎瞬间完成
- **文件同步**: 源文件和目标文件内容完全同步

#### 使用条件

1. **同一文件系统**: 源文件和目标必须在同一分区/NAS卷
2. **文件系统支持**: ext4、NTFS、APFS 等均支持
3. **权限要求**: 需要对源和目标目录有写入权限

#### 跨设备错误

如果源文件和目标不在同一文件系统，会显示错误：
```
硬链接失败：源文件和目标必须在同一文件系统
```

此时可选择：
- 将目标目录改为与源文件同一文件系统
- 使用复制模式 (`--mode=copy`)，或通过 `--fallback-on-link-failure=copy` 自动回退
- 使用移动模式 (`--mode=move`)，或通过 `--fallback-on-link-failure=move` 自动回退

### 🔧 刮削子命令（需 `--features scraper`）

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

### 📡 RSS 订阅管理（需 `--features clouddrive`）

```bash
# 列出订阅
aniorg rss --list-subscriptions

# 添加订阅
aniorg rss --add-subscription \
  --rss-url "https://example.com/rss" \
  --rss-target "/anime" \
  --rss-filter "720p"

# 单次执行
aniorg rss --single-shot \
  --clouddrive-url http://localhost:19798 \
  --rss-url "https://example.com/rss" \
  --rss-target "/anime"

# Daemon 模式（持续监控）
aniorg rss --daemon \
  --clouddrive-url http://localhost:19798 \
  --rss-interval 300
```

认证支持两种方式：直接传 `--clouddrive-token`，或使用 `--clouddrive-user` + `--clouddrive-pass` 登录获取令牌。订阅和去重状态默认保存在 Windows 的 `%LOCALAPPDATA%\anime-organizer\rss.db`。

### ☁️ CloudDrive 辅助命令（需 `--features clouddrive`）

```bash
# 直接提交 magnet 或 torrent URL 到离线下载
aniorg add-offline "magnet:?xt=urn:btih:..." \
  --target "/anime" \
  --clouddrive-url http://localhost:19798 \
  --clouddrive-token "YOUR_TOKEN"

# 浏览云盘目录
aniorg list-folder "/anime" \
  --clouddrive-url http://localhost:19798 \
  --clouddrive-token "YOUR_TOKEN"
```

### 💡 使用示例

```bash
# 整理下载目录
aniorg --source="D:\Downloads\Anime"

# 整理到指定目录
aniorg -s "/home/user/Downloads" -t "/media/anime"

# 硬链接模式
aniorg -s "/home/user/Downloads" -m link -t "/mnt/anime"

# 多季作品按 Season 目录整理
aniorg -s "/path/to/downloads" -t "/path/to/anime" --season-mode

# 预览变更并显示详细日志
aniorg -s "/path/to/downloads" --dry-run -v

# 指定文件类型
aniorg -s "/path/to/downloads" --include-ext="mp4,mkv"

# 使用自定义别名库
aniorg -s "/path/to/downloads" --alias-file="./my-aliases.json"

# 强制覆盖已有 NFO
aniorg -s "/path/to/downloads" --scrape-metadata --force-overwrite
```

---

## English

### 📖 Project Description

anime-organizer (aniorg) is a command-line tool designed for anime collectors to automatically organize video files in download directories.

**Core Features:**

- Auto-parse filenames to extract anime name, episode number, publisher info
- Support hard link mode - organize files without using extra disk space
- Generate Kodi-compatible NFO metadata files
- Download poster and fanart images
- RSS subscription and 115 cloud drive sync (requires clouddrive feature)

**Use Cases:**

- Organize anime files in download directories
- Automate anime media library management
- Generate metadata for Kodi/Jellyfin/Plex media centers

### 🚀 Features

- **Smart Parsing**: Auto-recognize `[Publisher] AnimeName - Episode [Tags].ext` format
- **Flexible Organization**: Restructure to `AnimeName/Season N/Episode [Tags].ext`
- **Multiple Modes**: Support move, copy, and hard link operations
- **Metadata Scraping**: Generate Kodi-compatible NFO files from Bangumi metadata
- **Artwork Download**: Download posters and fanart from TMDB with AniDB poster fallback
- **Alias Matching**: Local `bangumi.db` aliases plus optional custom JSON overrides, with Bangumi title/search fallback
- **RSS Subscription**: RSS auto-subscription and 115 cloud drive sync (requires `clouddrive` feature)
- **Cross-Platform**: Support Windows, Linux, macOS
- **Zero Runtime Dependencies**: Single binary deployment
- **High Performance**: Native Rust implementation

### 📥 Installation

#### Option 1: Download Pre-built Binary

Download from [GitHub Releases](https://github.com/ModerRAS/anime-organizer/releases).

#### Option 2: Build from Source

```bash
git clone https://github.com/ModerRAS/anime-organizer.git
cd anime-organizer

# Default build (metadata feature enabled)
cargo build --release

# Full features
cargo build --release --features "scraper clouddrive torrent-scraper"
```

### 🎯 Quick Start

```bash
# Hard link mode (default, zero extra space)
aniorg --source="/path/to/downloads"

# Preview mode
aniorg --source="/path/to/downloads" --dry-run --verbose

# Enable automatic fallback to copy when hard link fails
aniorg --source="/path/to/downloads" --fallback-on-link-failure=copy

# Organize multi-season releases into Season folders
aniorg --source="/path/to/downloads" --season-mode --target="/path/to/anime"

# Generate NFO files and artwork
aniorg --source="/path/to/downloads" --scrape-metadata --tmdb-api-key="YOUR_TMDB_KEY"
```

### 📋 Arguments

| Argument | Short | Type | Required | Default | Description |
|----------|-------|------|----------|---------|-------------|
| `--source` | `-s` | string | ✅ | - | Source directory path |
| `--target` | `-t` | string | ❌ | source | Target root directory |
| `--mode` | `-m` | enum | ❌ | link | Operation mode: move/copy/link |
| `--dry-run` | | bool | ❌ | false | Preview only, no actual changes |
| `--include-ext` | | string | ❌ | mp4,mkv,... | File extensions to process |
| `--verbose` | `-v` | bool | ❌ | false | Show detailed logs |
| `--fallback-on-link-failure` | | enum | ❌ | - | Fallback when hard link fails: move or copy (disabled by default) |
| `--season-mode` / `--分季` | | bool | ❌ | false | Group multi-season titles as `Series/Season N/` |
| `--scrape-metadata` / `--刮削` | | bool | ❌ | false | Enable Bangumi/TMDB metadata scraping |
| `--tmdb-api-key` | | string | ❌ | - | TMDB API key for artwork download |
| `--alias-file` | | string | ❌ | - | Custom alias JSON file overriding same-name entries from local `bangumi.db` |
| `--no-images` | | bool | ❌ | false | Generate NFO only, skip artwork download |
| `--force-overwrite` | | bool | ❌ | false | Overwrite existing NFO and image files |
| `--bangumi-cache` | | string | ❌ | system temp dir | Bangumi cache directory |
| `--metadata-source` | | string | ❌ | - | Local `subject.jsonlines` file or containing directory |
| `--help` | `-h` | bool | ❌ | false | Show help |
| `--version` | `-V` | bool | ❌ | false | Show version |

### 🧾 Metadata Scraping

When `--scrape-metadata` is enabled, the tool will:

- Prefer aliases from `--alias-file` JSON and a local `bangumi.db`; if neither is available, it falls back to Bangumi title/search matching
- Generate `tvshow.nfo` in the series root
- Generate per-episode `*.nfo` files beside organized videos in `Season N/`
- Download `poster.jpg`, `fanart.jpg`, and `seasonXX-poster.jpg` when a TMDB API key is provided

The `--alias-file` JSON can reuse the object output format from `extract-aliases`, for example:

```json
{
  "Frieren": {
    "bangumi_id": 444444,
    "name": "葬送のフリーレン",
    "tmdb_id": 209867,
    "anidb_id": 18597
  }
}
```

### 🔗 Hard Link Notes

Hard linking is the recommended mode:

- **Zero Extra Space**: No additional disk space used
- **Fast Operation**: Almost instant
- **File Sync**: Source and target share the same content

If hard linking fails due to cross-filesystem layouts or lack of support, you can opt in to automatic fallback via `--fallback-on-link-failure=copy` or `--fallback-on-link-failure=move`; otherwise, the failure is reported and the file is skipped.

**Requirements:**
1. Source and target must be on the same filesystem
2. Filesystem must support hard links (ext4, NTFS, APFS, etc.)
3. Write permission required for both directories

### 🔧 Scraper Subcommands (requires `--features scraper`)

```bash
# Scrape recent updates
cargo run --features scraper -- scrape --days 7 --format json

# Match alias proposals
cargo run --features scraper -- match --input scraped.json --format github

# Build SQLite alias database
cargo run --features scraper -- build-db --output bangumi.db

# Extract aliases from dump
cargo run --features scraper -- extract-aliases --download

# Merge new aliases into database
cargo run --features scraper -- merge-aliases --input new_aliases.json

# Apply confident match proposals
cargo run --features scraper -- apply-matches --input proposals.json

# Create alias request issues
cargo run --features scraper -- create-alias-issues --input uncertain.json --repo ModerRAS/anime-organizer
```

### 📡 RSS Subscription Management (requires `--features clouddrive`)

```bash
# List subscriptions
aniorg rss --list-subscriptions

# Add subscription
aniorg rss --add-subscription \
  --rss-url "https://example.com/rss" \
  --rss-target "/anime" \
  --rss-filter "720p"

# Single shot execution
aniorg rss --single-shot \
  --clouddrive-url http://localhost:19798 \
  --rss-url "https://example.com/rss" \
  --rss-target "/anime"

# Daemon mode (continuous monitoring)
aniorg rss --daemon \
  --clouddrive-url http://localhost:19798 \
  --rss-interval 300
```

Authentication supports either `--clouddrive-token` or `--clouddrive-user` + `--clouddrive-pass`. Subscription and dedupe state is stored in `%LOCALAPPDATA%\anime-organizer\rss.db` on Windows.

### ☁️ CloudDrive Helper Commands (requires `--features clouddrive`)

```bash
# Submit a magnet or torrent URL directly to offline download
aniorg add-offline "magnet:?xt=urn:btih:..." \
  --target "/anime" \
  --clouddrive-url http://localhost:19798 \
  --clouddrive-token "YOUR_TOKEN"

# Browse a cloud folder
aniorg list-folder "/anime" \
  --clouddrive-url http://localhost:19798 \
  --clouddrive-token "YOUR_TOKEN"
```

### 🧲 Torrent Scraping (requires `--features torrent-scraper`)

```bash
# Scrape DMHY torrent titles
aniorg torrent-scrape --source dmhy --output dmhy_titles.txt

# Scrape Nyaa torrent titles
aniorg torrent-scrape --source nyaa --query "anime" --output nyaa_titles.txt

# Scrape both sources
aniorg torrent-scrape --source all --pages 3 --output all_titles.txt
```

### 🛠️ Development

```bash
# Run tests
cargo test

# Run with verbose output
cargo run -- --source="/path/to/downloads" --verbose

# Build release binary
cargo build --release

# Build with all features
cargo build --release --features "scraper clouddrive torrent-scraper"
```

## License

AGPL-3.0 License - see [LICENSE](LICENSE) for details.
