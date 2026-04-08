# RSS/CloudDrive 模块知识库

**Generated:** 2026-04-08
**Parent:** `./AGENTS.md`

## OVERVIEW
RSS订阅监控和CloudDrive2离线下载功能，支持Daemon模式和单次执行。

## FILES
| File | Purpose |
|------|---------|
| `mod.rs` | 模块导出 |
| `client.rs` | CloudDrive2 gRPC客户端 |
| `db.rs` | RSS订阅+已处理项SQLite |
| `parser.rs` | RSS 2.0 XML解析 |
| `scheduler.rs` | 轮询调度、Daemon模式 |
| `processor.rs` | RSS项处理逻辑 |
| `filter.rs` | 正则过滤 |
| `http_client.rs` | HTTP抽象层 |
| `torrent.rs` | 种子解析 |
| `proxy.rs` | 代理配置 |

## KEY TYPES
```rust
pub struct RssSubscription { id, url, filter_regex, target_folder, interval_secs }
pub struct ProcessedItem { id, item_hash, url, ... }
pub struct RssItem { title, link, pub_date, enclosure }
```

## CLOUDDRIVE GRPC
- 生成的代码在`src/`（`build.rs`编译proto）
- 支持：115网盘/Xunlei/Aliyun/Baidu/OneDrive等
- JWT认证 + 登录刷新token

## RSS FLOW
1. 读取订阅列表（SQLite）
2. 抓取RSS Feed → `quick-xml`解析
3. 正则过滤匹配
4. 种子解析（`bt_bencode`）
5. CloudDrive2离线下载
6. 记录已处理项（去重）

## DAEMON MODE
```bash
aniorg rss --daemon --clouddrive-url http://localhost:19798 --rss-interval 300
```

## DATABASE SCHEMA
```sql
CREATE TABLE subscriptions (id PRIMARY KEY, url, filter_regex, target_folder, interval_secs);
CREATE TABLE processed_items (id PRIMARY KEY, item_hash, url, processed_at);
```

## PROXY CONFIG
- 从环境变量读取：`HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`
- 支持格式：`http://user:pass@host:port`

## ANTI-PATTERNS
- ❌ 不要在生产环境忽略RSS错误
- ❌ 不要跳过item_hash去重检查
- ❌ CloudDrive URL必须以http://或https://开头
