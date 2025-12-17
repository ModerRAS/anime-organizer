# anime-organizer

[![CI](https://github.com/ModerRAS/anime-organizer/actions/workflows/ci.yml/badge.svg)](https://github.com/ModerRAS/anime-organizer/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/anime-organizer.svg)](https://crates.io/crates/anime-organizer)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

轻量级、跨平台命令行工具，专为动漫收藏者批量整理视频文件，支持硬链接模式实现零额外空间占用。

[English](#english) | [中文](#中文)

---

## 中文

### 🚀 功能特性

- **智能解析**: 自动识别 `[发布组] 动漫名 - 集数 [标签].ext` 格式
- **灵活整理**: 重构为 `动漫名/集数 [标签].ext` 结构
- **多种模式**: 支持移动、复制、硬链接三种操作模式
- **跨平台**: 支持 Windows、Linux、macOS
- **零依赖运行**: 单文件部署，无需外部配置
- **高性能**: Rust 原生实现，极速处理

### 📥 安装方式

#### 方式一：从 Cargo 安装

```bash
cargo install anime-organizer
```

#### 方式二：下载预编译二进制

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

#### 方式三：从源码构建

```bash
git clone https://github.com/ModerRAS/anime-organizer.git
cd anime-organizer
cargo build --release
```

编译后的二进制文件位于 `target/release/aniorg`。

### 🎯 快速开始

#### 基本用法

```bash
# 移动模式（默认）
aniorg --source="/path/to/downloads"

# 硬链接模式（推荐，零额外空间）
aniorg --source="/path/to/downloads" --mode=link --target="/path/to/anime"

# 复制模式
aniorg --source="/path/to/downloads" --mode=copy --target="/path/to/anime"
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
| `--mode` | `-m` | enum | ❌ | move | 操作模式：move/copy/link |
| `--dry-run` | | bool | ❌ | false | 仅预览不执行 |
| `--include-ext` | | string | ❌ | mp4,mkv,... | 处理的扩展名（逗号分隔） |
| `--verbose` | `-v` | bool | ❌ | false | 显示详细日志 |
| `--help` | `-h` | bool | ❌ | false | 显示帮助 |
| `--version` | `-V` | bool | ❌ | false | 显示版本 |

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
├── 01 [标签信息].扩展名
├── 02 [标签信息].扩展名
└── ...
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
- 使用复制模式 (`--mode=copy`)
- 使用移动模式 (`--mode=move`)

### 💡 使用示例

```bash
# 整理下载目录
aniorg --source="D:\Downloads\Anime"

# 整理到指定目录
aniorg -s "/home/user/Downloads" -t "/media/anime"

# 硬链接模式
aniorg -s "/home/user/Downloads" -m link -t "/mnt/anime"

# 预览变更并显示详细日志
aniorg -s "/path/to/downloads" --dry-run -v

# 指定文件类型
aniorg -s "/path/to/downloads" --include-ext="mp4,mkv"
```

---

## English

### 🚀 Features

- **Smart Parsing**: Auto-recognize `[Publisher] AnimeName - Episode [Tags].ext` format
- **Flexible Organization**: Restructure to `AnimeName/Episode [Tags].ext`
- **Multiple Modes**: Support move, copy, and hard link operations
- **Cross-Platform**: Support Windows, Linux, macOS
- **Zero Runtime Dependencies**: Single binary deployment
- **High Performance**: Native Rust implementation

### 📥 Installation

#### Option 1: Install via Cargo

```bash
cargo install anime-organizer
```

#### Option 2: Download Pre-built Binary

Download from [GitHub Releases](https://github.com/ModerRAS/anime-organizer/releases).

#### Option 3: Build from Source

```bash
git clone https://github.com/ModerRAS/anime-organizer.git
cd anime-organizer
cargo build --release
```

### 🎯 Quick Start

```bash
# Move mode (default)
aniorg --source="/path/to/downloads"

# Hard link mode (recommended, zero extra space)
aniorg --source="/path/to/downloads" --mode=link --target="/path/to/anime"

# Preview mode
aniorg --source="/path/to/downloads" --dry-run --verbose
```

### 📋 Arguments

| Argument | Short | Type | Required | Default | Description |
|----------|-------|------|----------|---------|-------------|
| `--source` | `-s` | string | ✅ | - | Source directory path |
| `--target` | `-t` | string | ❌ | source | Target root directory |
| `--mode` | `-m` | enum | ❌ | move | Operation mode: move/copy/link |
| `--dry-run` | | bool | ❌ | false | Preview only, no actual changes |
| `--include-ext` | | string | ❌ | mp4,mkv,... | File extensions to process |
| `--verbose` | `-v` | bool | ❌ | false | Show detailed logs |
| `--help` | `-h` | bool | ❌ | false | Show help |
| `--version` | `-V` | bool | ❌ | false | Show version |

### 🔗 Hard Link Notes

Hard linking is the recommended mode:

- **Zero Extra Space**: No additional disk space used
- **Fast Operation**: Almost instant
- **File Sync**: Source and target share the same content

**Requirements:**
1. Source and target must be on the same filesystem
2. Filesystem must support hard links (ext4, NTFS, APFS, etc.)
3. Write permission required for both directories

### 🛠️ Development

```bash
# Run tests
cargo test

# Run with verbose output
cargo run -- --source="/path/to/downloads" --verbose

# Build release binary
cargo build --release
```

## License

AGPL-3.0 License - see [LICENSE](LICENSE) for details.
