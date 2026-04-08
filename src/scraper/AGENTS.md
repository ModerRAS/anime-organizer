# Scraper 模块知识库

**Generated:** 2026-04-08
**Parent:** `./AGENTS.md`

## OVERVIEW
从多个数据源定期采集动画信息，构建SQLite别名库，LLM辅助别名匹配。

## FILES
| File | Purpose |
|------|---------|
| `mod.rs` | 模块导出 |
| `sources.rs` | 多源刮削（Bangumi/TMDB/DMHY） |
| `matcher.rs` | LLM API别名提案生成 |
| `db_builder.rs` | SQLite数据库构建（Bangumi Archive ZIP解析） |

## KEY TYPES
```rust
pub struct BuildDbStats { subjects_count, episodes_count, aliases_count, ... }
pub struct ScrapedAnime { title, title_cn, date, source }
pub struct Proposal { fan_translation, alias_entry, confidence, reasoning }
```

## DATABASE SCHEMA
```sql
CREATE TABLE subjects (id PRIMARY KEY, type, name, name_cn, summary, date, score, ...);
CREATE TABLE aliases (id AUTOINCREMENT, subject_id, alias UNIQUE);
CREATE TABLE episodes (id, subject_id, sort, disc, type PRIMARY KEY);
CREATE TABLE subject_relations (subject_id, related_subject_id PRIMARY KEY);
```

## DATABASE BUILD FLOW
1. 下载Bangumi Archive ZIP → `flate2`解压
2. 解析`subject.jsonlines` → 批量INSERT（1000条/批）
3. 解析`episode.jsonlines` → 按subject_id过滤
4. 从name_cn提取别名 → `INSERT OR IGNORE`
5. 可选：relations/characters/persons

## SQL CONFLICT STRATEGY
- **DO UPDATE** for subjects (overwrite on conflict)
- **DO NOTHING** for episodes/aliases (preserve existing)
- 使用`unchecked_transaction()`提升性能

## LLM MATCHING
- 调用外部LLM API匹配fan translation → Bangumi ID
- 输出confidence (high/medium/low)
- 生成GitHub issue提案格式

## BUILD COMMANDS
```bash
cargo build --features scraper
cargo run --features scraper -- build-db --output bangumi.db --verbose
cargo run --features scraper -- scrape --days 7 --format json
cargo run --features scraper -- match --input scraped.json --format github
```

## ANTI-PATTERNS
- ❌ 不要在episode插入时用`DO UPDATE`（会覆盖已有记录）
- ❌ 不要跳过subject_id验证直接插入episode
