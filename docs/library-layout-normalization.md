# Library Layout Normalization Plan

## Status

Design only. This document does not authorize changes to `S:\动漫` and does not change current organize behavior.

The migration must not run until the daemon has no active organize job. A dry-run plan and conflict review are required before apply.

## Background

Three historical organizer changes created four physical layouts:

| Generation | Directory | Video filename | Example |
| --- | --- | --- | --- |
| Old flat | Parsed title, possibly including season | Generated episode filename | `Title S2/01 [1080p].mkv` |
| Old season | Series title without season / `Season N` | Generated episode filename | `Title/Season 2/01 [1080p].mkv` |
| Canonical | Series title without season / `Season N` | Original release filename | `Title/Season 2/[Group] Title S2 - 01 [...].mkv` |
| Bug flat | Parsed title, possibly including season | Original release filename | `Title S2/[Group] Title S2 - 01 [...].mkv` |

The 2026-07-29 inventory snapshot found:

- 1,734 old-flat videos.
- 398 old-season videos.
- 1,936 canonical videos.
- 1,452 bug-flat videos.
- 252 videos in other nested layouts, primarily disc extras and menus.

These counts are only a design snapshot. The planner must rescan after all active organize jobs finish.

## Canonical Layout

The only target layout managed by this feature is:

```text
<series title without season suffix>/Season N/<existing video basename>
```

For a video that still has its original release filename, the result is:

```text
Title/Season 2/[Group] Title S2 - 01 [1080p].mkv
```

The migration never renames a video merely to make an old generated filename look like an original filename. If an old organizer version already replaced the original basename with `01 [tags].mkv`, the original basename cannot be reconstructed unless an existing byte-identical original-name copy is available.

## Goals

1. Produce a deterministic, reviewable plan for normalizing the four known layouts.
2. Keep original release filenames whenever they still exist.
3. Delete exact duplicate videos while keeping the representation closest to the canonical layout.
4. Persist quick and full SHA-256 values in `library.db` so later scans do not reread unchanged files from `S:`.
5. Move matching subtitles and other episode sidecars with the selected video.
6. Rebuild or transactionally update `library.db` only after filesystem operations succeed.
7. Never overwrite or delete a file based on parsing, size, or a prefix hash alone.

## Non-goals

- Guessing lost original release filenames.
- Automatically restructuring unrecognized nested disc layouts, menus, NCOP, NCED, or other extras in the first release.
- Deduplicating byte-identical files that represent different logical series, seasons, or episodes.
- Silently falling back to a full copy when a same-library rename fails.
- Running concurrently with normal organize jobs.

## User Interface

Add a dedicated command instead of running ordinary organize with the same source and target:

```powershell
# Read-only inventory and plan
aniorg normalize-layout --target 'S:\动漫' --dry-run --plan layout-plan.json

# Apply one reviewed plan
aniorg normalize-layout --target 'S:\动漫' --apply-plan layout-plan.json
```

The daemon job type is `normalize_layout` with the same target and plan path. Applying a plan is destructive and requires the daemon request's top-level `confirmed=true`. Dry-run does not require confirmation.

A plan contains:

- Plan format version and creation time.
- Library root and `library.db` identity.
- Every source and destination relative path.
- Logical series, season, episode, and metadata authority used.
- Layout classification and keeper priority.
- File size, modified time, prefix SHA-256, and full SHA-256 when required.
- Planned action: `keep`, `move`, `deduplicate`, `conflict`, or `unresolved`.
- Companion-file actions.
- Summary counts and bytes by action.

Apply rejects the plan if the target root differs or a participating file no longer matches its planned size and modified time. It does not silently recompute a different plan.

## Layout Classification

Classification must use existing Rust parsers and library-index path interpretation, not a filename regex as the sole authority:

1. Parse full release filenames with `FilenameParser`.
2. Parse generated episode-only filenames through the existing legacy target-path logic.
3. Interpret an explicit `Season N` parent when present.
4. Use existing `library.db` Bangumi IDs and episode records when available.
5. Report paths that cannot be assigned one unambiguous series, season, and episode.

The 252 currently observed videos in other nested layouts remain untouched unless a later feature defines an explicit extras migration policy.

## Logical Identity

Deduplication requires the same logical identity as well as identical content.

Preferred identity key:

```text
Bangumi subject ID + season + episode
```

Fallback identity key when no authoritative ID exists:

```text
normalized series title + season + episode
```

A byte-identical file with a different logical identity is retained. This prevents deletion of intentionally reused openings, specials, recaps, or incorrectly parsed episodes.

## Hash Records

Extend `media_file` in `library.db` with nullable lowercase hexadecimal fields:

```sql
sha256_prefix_63m TEXT CHECK (
    sha256_prefix_63m IS NULL OR length(sha256_prefix_63m) = 64
),
sha256_full TEXT CHECK (
    sha256_full IS NULL OR length(sha256_full) = 64
)
```

Existing `size` and `modified_time` are the cache identity for these hashes.

Hash definitions:

- `sha256_prefix_63m`: SHA-256 over bytes `[0, min(file_size, 63 * 1024 * 1024))`.
- `sha256_full`: SHA-256 over the complete file.

Rules:

1. Inventory reads `size` and `modified_time` first.
2. An existing hash is reused when the stored path, size, and modified time match.
3. A controlled move performed by the migration carries both hashes to the new path without rereading the file.
4. An upsert that observes changed size or modified time clears both cached hashes before storing new values.
5. A full `library.db` rebuild imports hashes from the previous database for unchanged path/size/modified-time tuples.
6. Prefix SHA-256 is computed for inventory and candidate grouping.
7. Full SHA-256 is computed and persisted before any duplicate deletion.
8. Matching size and prefix SHA-256 are never sufficient for deletion.

CloudDrive can expose stale timestamps. The cache assumes media files are immutable unless replaced through anime-organizer. A file modified externally while retaining the same path, size, and timestamp cannot be detected without rereading it; destructive apply should offer a force-rehash option for such libraries, but it is not the default because it defeats the persistent cache.

## Duplicate Selection

Candidate grouping uses logical identity, file size, and `sha256_prefix_63m`. Candidate groups then receive full SHA-256 values. Files are duplicates only when logical identity, size, and `sha256_full` all match.

Keeper priority, highest first:

1. Canonical Season directory plus original release filename.
2. Flat directory plus original release filename.
3. Canonical Season directory plus generated episode filename.
4. Flat directory plus generated episode filename.

For example, when these are byte-identical:

```text
Title/Season 2/01 [1080p].mkv
Title S2/[Group] Title S2 - 01 [1080p].mkv
```

keep the original-name video, move it unchanged into `Title/Season 2/`, then delete the generated-name duplicate. The video basename is never rewritten.

Equal-priority ties prefer an already canonical path, then the lexicographically smallest relative path for deterministic plans.

Different hashes for the same logical episode are treated as separate releases and retained.

## Companion Files

For each selected video, collect supported subtitles and episode-owned sidecars sharing its stem.

- Preserve unique subtitles, NFO files, and thumbnails.
- If the keeper has an original release basename and a deleted generated-name duplicate owns the only subtitle, move the subtitle to the keeper directory and adjust the subtitle stem to match the keeper video.
- If two sidecars target the same path, compare full content before deleting either one.
- Conflicting sidecars are reported and left untouched.
- Series artwork is merged into the canonical series root only when the target is absent or byte-identical.

The no-video-rename rule does not prohibit renaming a sidecar when necessary to remain attached to the selected keeper.

## Planning Algorithm

1. Acquire the daemon's exclusive library resource and verify no organize job is running.
2. Copy `library.db` locally and validate `PRAGMA integrity_check`.
3. Snapshot all recognized videos and sidecars without mutating the filesystem.
4. Load metadata identity and cached hashes from the local database copy.
5. Classify each path and compute its canonical directory.
6. Compute missing prefix hashes with 1 MiB streaming buffers, reading at most 63 MiB per file.
7. Group possible duplicates by logical identity, size, and prefix hash.
8. Compute or reuse full SHA-256 for members of duplicate candidate groups.
9. Select keepers and produce all move, deduplication, conflict, and unresolved actions.
10. Write the immutable JSON plan and a concise human-readable summary.

Planning is read-only. It must not create Season directories, move files, delete duplicates, or replace `library.db`.

## Apply Algorithm

1. Reacquire the exclusive library resource and require `confirmed=true`.
2. Revalidate plan version, library root, source existence, size, modified time, and cached hash identity.
3. Ensure each selected keeper reaches its canonical directory using same-library `rename` only.
4. Move or merge keeper sidecars.
5. Verify the keeper exists at the planned destination.
6. Delete an older duplicate only after its full SHA-256 equals the keeper's full SHA-256 and logical identity still matches.
7. Move remaining nonduplicate legacy videos into canonical Season directories without changing their basenames.
8. Leave every conflict and unresolved path untouched.
9. Remove empty legacy directories, but never remove a directory containing unplanned files.
10. Update paths and hashes in a local database transaction, or perform a full local rebuild while importing valid hash cache rows.
11. Run `PRAGMA integrity_check`, validate referenced media paths, publish `library.db`, and ensure no staging files remain.

If a same-library rename fails, record an error and stop that action. Do not silently copy hundreds of gigabytes through CloudDrive.

## Failure and Recovery

- Every action is idempotent and records its completion in the job log.
- An existing planned destination is accepted only when it is the planned keeper or is full-hash identical.
- A destination with different content becomes a conflict; it is never overwritten.
- Filesystem success followed by database failure is recoverable by rebuilding `library.db` from the final filesystem and importing hashes from the plan.
- Database publication happens only after all attempted filesystem actions finish and validation succeeds.
- The original reviewed plan remains available as an audit artifact.

## Verification

The implementation requires focused tests for:

1. All four historical layouts map to the canonical directory.
2. Original video basenames are preserved.
3. Lost original basenames are not guessed.
4. Keeper priority selects the newest representation without renaming the video.
5. Prefix equality without full equality never deletes a file.
6. Full-hash equality with different logical identity never deletes a file.
7. Different releases of one episode are both retained.
8. Unique subtitles from a deleted duplicate remain attached to the keeper.
9. Conflicts do not overwrite either file.
10. Cached hashes survive controlled moves and database rebuilds.
11. Changed size or modified time invalidates cached hashes.
12. Dry-run does not modify the filesystem or `library.db`.
13. Apply requires daemon confirmation.
14. Interrupted apply can be rerun safely.

Repository checks remain mandatory:

```bash
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --document-private-items
```

## Rollout

1. Finish and verify the currently running download cleanup job before any normalization scan.
2. Release schema/hash-cache support and the read-only planner.
3. Generate a fresh plan for `S:\动漫`; review counts, unresolved paths, conflicts, and expected reclaimed bytes.
4. Back up `library.db` and apply the reviewed plan through one confirmed daemon job.
5. Verify file counts, representative paths from all four generations, database integrity, media references, hash population, and absence of staging files.
6. Keep the plan and job logs as the migration audit record.

No production file under `S:\动漫` is changed as part of approving this plan.
