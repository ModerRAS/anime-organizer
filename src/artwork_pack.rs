use crate::error::{AppError, Result};
use crate::library_index::{Artwork, ArtworkKind, LibraryIndexRecord, DATABASE_FILENAME};
use rusqlite::{params, Connection};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) const ARTWORK_DIRECTORY: &str = "MLIP-Artwork";
pub(crate) const TARGET_PACK_BYTES: u64 = 64 * 1024 * 1024;
pub(crate) const MAX_PACK_BYTES: u64 = 96 * 1024 * 1024;
const TAR_BLOCK: u64 = 512;
const TAR_TRAILER_BYTES: u64 = TAR_BLOCK * 2;
const MAX_IMAGE_DIMENSION: u32 = 32_768;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct ImageCandidate {
    source_path: PathBuf,
    sha256: String,
    extension: String,
    media_type: String,
    width: u32,
    height: u32,
    byte_length: u64,
}

#[derive(Debug, Clone)]
struct ArtworkPack {
    sha256: String,
    path: String,
    byte_length: u64,
    asset_count: usize,
}

#[derive(Debug, Clone)]
struct ArtworkAsset {
    sha256: String,
    pack_sha256: String,
    member_name: String,
    data_offset: u64,
    byte_length: u64,
    media_type: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Default)]
pub(crate) struct ArtworkPacking {
    packs: Vec<ArtworkPack>,
    assets: Vec<ArtworkAsset>,
    binding_assets: HashMap<Artwork, String>,
}

impl ArtworkPacking {
    pub(crate) fn write_catalog(&self, conn: &Connection) -> Result<HashMap<String, i64>> {
        let mut pack_ids = HashMap::new();
        let mut packs = self.packs.iter().collect::<Vec<_>>();
        packs.sort_by(|left, right| left.sha256.cmp(&right.sha256));
        for pack in packs {
            conn.execute(
                "INSERT INTO artwork_pack (sha256, path, byte_length, asset_count) \
                 VALUES (?1, ?2, ?3, ?4) \
                 ON CONFLICT(sha256) DO UPDATE SET \
                 path = excluded.path, byte_length = excluded.byte_length, \
                 asset_count = excluded.asset_count",
                params![
                    pack.sha256,
                    pack.path,
                    to_i64(pack.byte_length, "pack byte_length")?,
                    to_i64(pack.asset_count as u64, "pack asset_count")?,
                ],
            )
            .map_err(|error| catalog_error("写入 artwork_pack", error))?;
            let pack_id: i64 = conn
                .query_row(
                    "SELECT id FROM artwork_pack WHERE sha256 = ?1",
                    params![pack.sha256],
                    |row| row.get(0),
                )
                .map_err(|error| catalog_error("读取 artwork_pack id", error))?;
            pack_ids.insert(pack.sha256.clone(), pack_id);
        }

        let mut asset_ids = HashMap::new();
        let mut assets = self.assets.iter().collect::<Vec<_>>();
        assets.sort_by(|left, right| left.sha256.cmp(&right.sha256));
        for asset in assets {
            let pack_id = pack_ids.get(&asset.pack_sha256).ok_or_else(|| {
                AppError::LibraryIndexError(format!(
                    "artwork_asset {} 引用了未知 pack {}",
                    asset.sha256, asset.pack_sha256
                ))
            })?;
            conn.execute(
                "INSERT INTO artwork_asset \
                 (sha256, pack_id, member_name, data_offset, byte_length, media_type, width, height) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
                 ON CONFLICT(sha256) DO UPDATE SET \
                 pack_id = excluded.pack_id, member_name = excluded.member_name, \
                 data_offset = excluded.data_offset, byte_length = excluded.byte_length, \
                 media_type = excluded.media_type, width = excluded.width, height = excluded.height",
                params![
                    asset.sha256,
                    pack_id,
                    asset.member_name,
                    to_i64(asset.data_offset, "asset data_offset")?,
                    to_i64(asset.byte_length, "asset byte_length")?,
                    asset.media_type,
                    i64::from(asset.width),
                    i64::from(asset.height),
                ],
            )
            .map_err(|error| catalog_error("写入 artwork_asset", error))?;
            let asset_id: i64 = conn
                .query_row(
                    "SELECT id FROM artwork_asset WHERE sha256 = ?1",
                    params![asset.sha256],
                    |row| row.get(0),
                )
                .map_err(|error| catalog_error("读取 artwork_asset id", error))?;
            asset_ids.insert(asset.sha256.clone(), asset_id);
        }
        Ok(asset_ids)
    }

    pub(crate) fn asset_id(
        &self,
        artwork: &Artwork,
        asset_ids: &HashMap<String, i64>,
    ) -> Option<i64> {
        self.binding_assets
            .get(artwork)
            .and_then(|sha256| asset_ids.get(sha256))
            .copied()
    }
}

pub(crate) fn build_and_publish(
    target_root: &Path,
    records: &[LibraryIndexRecord],
) -> Result<ArtworkPacking> {
    let artwork = records
        .iter()
        .flat_map(|record| {
            record
                .series_artwork
                .iter()
                .chain(record.episode_artwork.iter())
        })
        .cloned()
        .collect::<Vec<_>>();
    build_and_publish_artwork(target_root, &artwork)
}

pub(crate) fn build_and_publish_artwork(
    target_root: &Path,
    artwork: &[Artwork],
) -> Result<ArtworkPacking> {
    let (candidates, binding_assets) = collect_candidates(target_root, artwork)?;
    let needed_hashes = binding_assets.values().cloned().collect::<BTreeSet<_>>();
    let prior = load_prior_catalog(&target_root.join(DATABASE_FILENAME))?;
    let mut packs = Vec::new();
    let mut assets = Vec::new();
    let mut reused_hashes = BTreeSet::new();

    if let Some(prior) = prior {
        let selected_packs = prior
            .assets
            .iter()
            .filter(|asset| needed_hashes.contains(&asset.sha256))
            .map(|asset| asset.pack_sha256.clone())
            .collect::<BTreeSet<_>>();
        for pack_sha256 in selected_packs {
            let pack = prior
                .packs
                .iter()
                .find(|pack| pack.sha256 == pack_sha256)
                .ok_or_else(|| {
                    AppError::LibraryIndexError(format!(
                        "旧 artwork_asset 引用了缺失 pack {pack_sha256}"
                    ))
                })?;
            let pack_assets = prior
                .assets
                .iter()
                .filter(|asset| asset.pack_sha256 == pack_sha256)
                .cloned()
                .collect::<Vec<_>>();
            verify_pack(target_root, pack, &pack_assets)?;
            reused_hashes.extend(pack_assets.iter().map(|asset| asset.sha256.clone()));
            packs.push(pack.clone());
            assets.extend(pack_assets);
        }
    }

    let new_candidates = candidates
        .values()
        .filter(|candidate| !reused_hashes.contains(&candidate.sha256))
        .cloned()
        .collect::<Vec<_>>();
    for shard in shard_candidates(new_candidates) {
        let (pack, pack_assets) = write_and_publish_pack(target_root, &shard)?;
        packs.push(pack);
        assets.extend(pack_assets);
    }

    packs.sort_by(|left, right| left.sha256.cmp(&right.sha256));
    packs.dedup_by(|left, right| left.sha256 == right.sha256);
    assets.sort_by(|left, right| left.sha256.cmp(&right.sha256));
    assets.dedup_by(|left, right| left.sha256 == right.sha256);

    for pack in &packs {
        let pack_assets = assets
            .iter()
            .filter(|asset| asset.pack_sha256 == pack.sha256)
            .cloned()
            .collect::<Vec<_>>();
        verify_pack(target_root, pack, &pack_assets)?;
    }

    Ok(ArtworkPacking {
        packs,
        assets,
        binding_assets,
    })
}

fn collect_candidates(
    target_root: &Path,
    artwork: &[Artwork],
) -> Result<(BTreeMap<String, ImageCandidate>, HashMap<Artwork, String>)> {
    let mut artwork = artwork.to_vec();
    artwork.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.kind.as_i64().cmp(&right.kind.as_i64()))
    });
    artwork.dedup();

    let mut candidates = BTreeMap::new();
    let mut binding_assets = HashMap::new();
    for item in artwork {
        if !item.has_valid_source_identity() {
            continue;
        }
        let Some(source_path) = safe_artwork_path(target_root, &item.path) else {
            continue;
        };
        let bytes = match std::fs::read(&source_path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let Some(image) = decode_image_metadata(&bytes) else {
            continue;
        };
        if matches!(item.kind, ArtworkKind::Poster | ArtworkKind::SeasonPoster)
            && image.width > image.height
        {
            continue;
        }
        let sha256 = sha256_hex(&bytes);
        let candidate = ImageCandidate {
            source_path,
            sha256: sha256.clone(),
            extension: image.extension.to_string(),
            media_type: image.media_type.to_string(),
            width: image.width,
            height: image.height,
            byte_length: bytes.len() as u64,
        };
        candidates
            .entry(sha256.clone())
            .and_modify(|existing: &mut ImageCandidate| {
                if candidate.source_path < existing.source_path {
                    existing.source_path = candidate.source_path.clone();
                }
            })
            .or_insert(candidate);
        binding_assets.insert(item, sha256);
    }
    Ok((candidates, binding_assets))
}

fn safe_artwork_path(target_root: &Path, relative: &str) -> Option<PathBuf> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(target_root.join(path))
}

#[derive(Debug)]
struct ImageMetadata {
    extension: &'static str,
    media_type: &'static str,
    width: u32,
    height: u32,
}

fn decode_image_metadata(bytes: &[u8]) -> Option<ImageMetadata> {
    let metadata = decode_png(bytes)
        .or_else(|| decode_jpeg(bytes))
        .or_else(|| decode_webp(bytes))?;
    let decoded = image::load_from_memory(bytes).ok()?;
    (metadata.width > 0
        && metadata.height > 0
        && metadata.width <= MAX_IMAGE_DIMENSION
        && metadata.height <= MAX_IMAGE_DIMENSION
        && decoded.width() == metadata.width
        && decoded.height() == metadata.height)
        .then_some(metadata)
}

fn decode_png(bytes: &[u8]) -> Option<ImageMetadata> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }
    let mut offset = 8_usize;
    let mut dimensions = None;
    let mut has_data = false;
    let mut has_end = false;
    while offset.checked_add(12)? <= bytes.len() {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
        let chunk_end = offset.checked_add(12)?.checked_add(length)?;
        if chunk_end > bytes.len() {
            return None;
        }
        let kind = &bytes[offset + 4..offset + 8];
        let data = &bytes[offset + 8..offset + 8 + length];
        let expected_crc =
            u32::from_be_bytes(bytes[offset + 8 + length..chunk_end].try_into().ok()?);
        if crc32(&bytes[offset + 4..offset + 8 + length]) != expected_crc {
            return None;
        }
        match kind {
            b"IHDR"
                if offset == 8
                    && length == 13
                    && valid_png_color_type(data[8], data[9])
                    && data[10] == 0
                    && data[11] == 0
                    && data[12] <= 1 =>
            {
                dimensions = Some((
                    u32::from_be_bytes(data[0..4].try_into().ok()?),
                    u32::from_be_bytes(data[4..8].try_into().ok()?),
                ));
            }
            b"IDAT" if !data.is_empty() => has_data = true,
            b"IEND" if length == 0 => {
                has_end = true;
                offset = chunk_end;
                break;
            }
            _ => {}
        }
        offset = chunk_end;
    }
    let (width, height) = dimensions?;
    (has_data && has_end && offset == bytes.len()).then_some(ImageMetadata {
        extension: "png",
        media_type: "image/png",
        width,
        height,
    })
}

fn valid_png_color_type(bit_depth: u8, color_type: u8) -> bool {
    matches!(
        (bit_depth, color_type),
        (1 | 2 | 4 | 8 | 16, 0) | (8 | 16, 2) | (1 | 2 | 4 | 8, 3) | (8 | 16, 4 | 6)
    )
}

fn decode_jpeg(bytes: &[u8]) -> Option<ImageMetadata> {
    if bytes.len() < 4 || !bytes.starts_with(&[0xff, 0xd8]) || !bytes.ends_with(&[0xff, 0xd9]) {
        return None;
    }
    let mut offset = 2_usize;
    let mut dimensions = None;
    let mut has_scan = false;
    while offset + 1 < bytes.len() - 2 {
        if bytes[offset] != 0xff {
            return None;
        }
        while offset < bytes.len() && bytes[offset] == 0xff {
            offset += 1;
        }
        let marker = *bytes.get(offset)?;
        offset += 1;
        if marker == 0xda {
            has_scan = true;
            break;
        }
        if marker == 0x01 || (0xd0..=0xd9).contains(&marker) {
            continue;
        }
        let segment_length =
            u16::from_be_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?) as usize;
        if segment_length < 2 || offset.checked_add(segment_length)? > bytes.len() - 2 {
            return None;
        }
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            if segment_length < 7 {
                return None;
            }
            let height = u16::from_be_bytes(bytes[offset + 3..offset + 5].try_into().ok()?) as u32;
            let width = u16::from_be_bytes(bytes[offset + 5..offset + 7].try_into().ok()?) as u32;
            dimensions = Some((width, height));
        }
        offset += segment_length;
    }
    let (width, height) = dimensions?;
    has_scan.then_some(ImageMetadata {
        extension: "jpg",
        media_type: "image/jpeg",
        width,
        height,
    })
}

fn decode_webp(bytes: &[u8]) -> Option<ImageMetadata> {
    if bytes.len() < 30 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    let riff_length = u32::from_le_bytes(bytes[4..8].try_into().ok()?) as usize + 8;
    if riff_length != bytes.len() {
        return None;
    }
    let mut offset = 12_usize;
    while offset.checked_add(8)? <= bytes.len() {
        let kind = &bytes[offset..offset + 4];
        let length = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?) as usize;
        let data_start = offset + 8;
        let data_end = data_start.checked_add(length)?;
        if data_end > bytes.len() {
            return None;
        }
        let data = &bytes[data_start..data_end];
        let dimensions = match kind {
            b"VP8L" if data.len() >= 5 && data[0] == 0x2f => {
                let bits = u32::from_le_bytes(data[1..5].try_into().ok()?);
                Some(((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1))
            }
            b"VP8 " if data.len() >= 10 && data[3..6] == [0x9d, 0x01, 0x2a] => Some((
                u16::from_le_bytes(data[6..8].try_into().ok()?) as u32 & 0x3fff,
                u16::from_le_bytes(data[8..10].try_into().ok()?) as u32 & 0x3fff,
            )),
            _ => None,
        };
        if let Some((width, height)) = dimensions {
            return Some(ImageMetadata {
                extension: "webp",
                media_type: "image/webp",
                width,
                height,
            });
        }
        offset = data_end.checked_add(length % 2)?;
    }
    None
}

fn shard_candidates(mut candidates: Vec<ImageCandidate>) -> Vec<Vec<ImageCandidate>> {
    candidates.sort_by(|left, right| left.sha256.cmp(&right.sha256));
    let mut shards = Vec::new();
    let mut shard = Vec::new();
    let mut size = TAR_TRAILER_BYTES;
    for candidate in candidates {
        let member_size = TAR_BLOCK + align_tar(candidate.byte_length);
        if !shard.is_empty() && size.saturating_add(member_size) > TARGET_PACK_BYTES {
            shards.push(std::mem::take(&mut shard));
            size = TAR_TRAILER_BYTES;
        }
        size = size.saturating_add(member_size);
        shard.push(candidate);
    }
    if !shard.is_empty() {
        shards.push(shard);
    }
    shards
}

fn write_and_publish_pack(
    target_root: &Path,
    candidates: &[ImageCandidate],
) -> Result<(ArtworkPack, Vec<ArtworkAsset>)> {
    let local_path = temp_path("artwork-pack", "tar");
    let result = (|| {
        let mut file = File::create(&local_path).map_err(pack_io("创建 artwork pack"))?;
        let mut assets = Vec::with_capacity(candidates.len());
        let mut offset = 0_u64;
        for candidate in candidates {
            let member_name = format!("{}.{}", candidate.sha256, candidate.extension);
            let header = tar_header(&member_name, candidate.byte_length)?;
            file.write_all(&header)
                .map_err(pack_io("写入 tar header"))?;
            offset += TAR_BLOCK;
            let data_offset = offset;
            let mut source = File::open(&candidate.source_path).map_err(pack_io("读取 artwork"))?;
            let copied =
                std::io::copy(&mut source, &mut file).map_err(pack_io("写入 artwork member"))?;
            if copied != candidate.byte_length {
                return Err(AppError::LibraryIndexError(format!(
                    "artwork 在打包期间发生变化: {}",
                    candidate.source_path.display()
                )));
            }
            let padding = align_tar(candidate.byte_length) - candidate.byte_length;
            write_zeros(&mut file, padding)?;
            offset += candidate.byte_length + padding;
            assets.push(ArtworkAsset {
                sha256: candidate.sha256.clone(),
                pack_sha256: String::new(),
                member_name,
                data_offset,
                byte_length: candidate.byte_length,
                media_type: candidate.media_type.clone(),
                width: candidate.width,
                height: candidate.height,
            });
        }
        write_zeros(&mut file, TAR_TRAILER_BYTES)?;
        file.sync_all().map_err(pack_io("同步 artwork pack"))?;
        drop(file);

        let byte_length = std::fs::metadata(&local_path)
            .map_err(pack_io("读取 artwork pack 大小"))?
            .len();
        if byte_length > MAX_PACK_BYTES
            && !(candidates.len() == 1
                && TAR_BLOCK + align_tar(candidates[0].byte_length) + TAR_TRAILER_BYTES
                    == byte_length)
        {
            return Err(AppError::LibraryIndexError(format!(
                "artwork pack 超过 96 MiB 限制: {byte_length} bytes"
            )));
        }
        let sha256 = sha256_file(&local_path)?;
        for asset in &mut assets {
            asset.pack_sha256.clone_from(&sha256);
        }
        let relative_path = format!("{ARTWORK_DIRECTORY}/{sha256}.tar");
        let pack = ArtworkPack {
            sha256,
            path: relative_path,
            byte_length,
            asset_count: assets.len(),
        };
        verify_pack_contents(&local_path, &pack, &assets)?;
        publish_pack(target_root, &local_path, &pack)?;
        Ok((pack, assets))
    })();
    let _ = std::fs::remove_file(&local_path);
    result
}

fn publish_pack(target_root: &Path, local_path: &Path, pack: &ArtworkPack) -> Result<()> {
    let directory = target_root.join(ARTWORK_DIRECTORY);
    std::fs::create_dir_all(&directory).map_err(pack_io("创建 MLIP-Artwork 目录"))?;
    let final_path = target_root.join(&pack.path);
    if final_path.exists() {
        return verify_pack_file(&final_path, pack);
    }

    let upload = directory.join(format!(
        ".{}.{}.tmp",
        pack.sha256,
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::copy(local_path, &upload).map_err(pack_io("上传 artwork pack"))?;
    if let Err(error) = verify_pack_file(&upload, pack) {
        let _ = std::fs::remove_file(&upload);
        return Err(error);
    }
    match std::fs::rename(&upload, &final_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::Unsupported => {
            match copy_new(local_path, &final_path) {
                Ok(()) => {}
                Err(copy_error)
                    if copy_error.kind() == std::io::ErrorKind::AlreadyExists
                        && verify_pack_file(&final_path, pack).is_ok() =>
                {
                    let _ = std::fs::remove_file(&upload);
                    return Ok(());
                }
                Err(copy_error) => {
                    let _ = std::fs::remove_file(&upload);
                    return Err(AppError::LibraryIndexError(format!(
                        "CloudDrive 不支持 pack rename 且复制失败: {copy_error}"
                    )));
                }
            }
        }
        Err(_) if final_path.exists() && verify_pack_file(&final_path, pack).is_ok() => {
            let _ = std::fs::remove_file(&upload);
            return Ok(());
        }
        Err(error) => {
            let _ = std::fs::remove_file(&upload);
            return Err(AppError::LibraryIndexError(format!(
                "发布 artwork pack 失败: {error}"
            )));
        }
    }
    let _ = std::fs::remove_file(&upload);
    if let Err(error) = verify_pack_file(&final_path, pack) {
        let _ = std::fs::remove_file(&final_path);
        return Err(error);
    }
    Ok(())
}

fn copy_new(source: &Path, destination: &Path) -> std::io::Result<()> {
    let mut source = File::open(source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let result = std::io::copy(&mut source, &mut output).and_then(|_| output.sync_all());
    if result.is_err() {
        drop(output);
        let _ = std::fs::remove_file(destination);
    }
    result
}

fn verify_pack(target_root: &Path, pack: &ArtworkPack, assets: &[ArtworkAsset]) -> Result<()> {
    if pack.asset_count != assets.len() {
        return Err(AppError::LibraryIndexError(format!(
            "artwork pack {} asset_count 不匹配",
            pack.sha256
        )));
    }
    let path = safe_pack_path(target_root, pack)?;
    verify_pack_contents(&path, pack, assets)
}

fn verify_pack_contents(path: &Path, pack: &ArtworkPack, assets: &[ArtworkAsset]) -> Result<()> {
    if pack.asset_count != assets.len() || (pack.byte_length > MAX_PACK_BYTES && assets.len() != 1)
    {
        return Err(AppError::LibraryIndexError(format!(
            "artwork pack {} size/asset_count 不匹配",
            pack.sha256
        )));
    }
    let mut ordered = assets.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|asset| asset.data_offset);
    let mut expected_header_offset = 0_u64;
    for asset in &ordered {
        if asset.data_offset != expected_header_offset + TAR_BLOCK {
            return Err(AppError::LibraryIndexError(format!(
                "artwork pack {} member offset 不连续",
                pack.sha256
            )));
        }
        expected_header_offset = asset.data_offset + align_tar(asset.byte_length);
    }
    if expected_header_offset.saturating_add(TAR_TRAILER_BYTES) != pack.byte_length {
        return Err(AppError::LibraryIndexError(format!(
            "artwork pack {} 长度与 catalog 不匹配",
            pack.sha256
        )));
    }

    verify_pack_file(path, pack)?;
    let mut file = File::open(path).map_err(pack_io("打开 artwork pack"))?;
    for asset in ordered {
        if asset.pack_sha256 != pack.sha256
            || asset.data_offset < TAR_BLOCK
            || asset.data_offset % TAR_BLOCK != 0
            || asset.member_name.contains('/')
            || asset.member_name.contains('\\')
            || !asset.member_name.starts_with(&asset.sha256)
        {
            return Err(AppError::LibraryIndexError(format!(
                "artwork pack {} catalog 非法",
                pack.sha256
            )));
        }
        file.seek(SeekFrom::Start(asset.data_offset - TAR_BLOCK))
            .map_err(pack_io("定位 tar header"))?;
        let mut header = [0_u8; 512];
        file.read_exact(&mut header)
            .map_err(pack_io("读取 tar header"))?;
        let name = tar_string(&header[0..100]);
        let size = tar_octal(&header[124..136])?;
        if name != asset.member_name
            || size != asset.byte_length
            || !matches!(header[156], 0 | b'0')
            || &header[257..265] != b"ustar\x0000"
            || tar_checksum(&header) != tar_octal(&header[148..156])?
        {
            return Err(AppError::LibraryIndexError(format!(
                "artwork pack {} member {} 无效",
                pack.sha256, asset.member_name
            )));
        }
        let length: usize = asset.byte_length.try_into().map_err(|_| {
            AppError::LibraryIndexError(format!("artwork pack {} member 太大", pack.sha256))
        })?;
        let mut bytes = vec![0_u8; length];
        file.read_exact(&mut bytes)
            .map_err(pack_io("读取 artwork member"))?;
        let actual = sha256_hex(&bytes);
        if actual != asset.sha256 {
            return Err(AppError::LibraryIndexError(format!(
                "artwork pack {} member {} SHA-256 不匹配",
                pack.sha256, asset.member_name
            )));
        }
        let metadata = decode_image_metadata(&bytes).ok_or_else(|| {
            AppError::LibraryIndexError(format!(
                "artwork pack {} member {} 不是有效图片",
                pack.sha256, asset.member_name
            ))
        })?;
        if asset.member_name != format!("{}.{}", asset.sha256, metadata.extension)
            || asset.media_type != metadata.media_type
            || asset.width != metadata.width
            || asset.height != metadata.height
        {
            return Err(AppError::LibraryIndexError(format!(
                "artwork pack {} member {} metadata 不匹配",
                pack.sha256, asset.member_name
            )));
        }
    }
    file.seek(SeekFrom::Start(expected_header_offset))
        .map_err(pack_io("定位 tar trailer"))?;
    let mut trailer = [1_u8; TAR_TRAILER_BYTES as usize];
    file.read_exact(&mut trailer)
        .map_err(pack_io("读取 tar trailer"))?;
    if trailer.iter().any(|byte| *byte != 0) {
        return Err(AppError::LibraryIndexError(format!(
            "artwork pack {} tar trailer 无效",
            pack.sha256
        )));
    }
    Ok(())
}

fn safe_pack_path(target_root: &Path, pack: &ArtworkPack) -> Result<PathBuf> {
    let expected = format!("{ARTWORK_DIRECTORY}/{}.tar", pack.sha256);
    if pack.path != expected || pack.sha256.len() != 64 || !is_lower_hex(&pack.sha256) {
        return Err(AppError::LibraryIndexError(format!(
            "非法 artwork pack 路径: {}",
            pack.path
        )));
    }
    Ok(target_root
        .join(ARTWORK_DIRECTORY)
        .join(format!("{}.tar", pack.sha256)))
}

fn verify_pack_file(path: &Path, pack: &ArtworkPack) -> Result<()> {
    let length = std::fs::metadata(path)
        .map_err(pack_io("读取 artwork pack"))?
        .len();
    if length != pack.byte_length || sha256_file(path)? != pack.sha256 {
        return Err(AppError::LibraryIndexError(format!(
            "artwork pack 缺失或损坏: {}",
            path.display()
        )));
    }
    Ok(())
}

#[derive(Default)]
struct PriorCatalog {
    packs: Vec<ArtworkPack>,
    assets: Vec<ArtworkAsset>,
}

fn load_prior_catalog(db_path: &Path) -> Result<Option<PriorCatalog>> {
    if !db_path.exists() {
        return Ok(None);
    }
    let conn =
        Connection::open(db_path).map_err(|error| catalog_error("打开旧 library.db", error))?;
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| catalog_error("读取旧 schema", error))?;
    if version != 4 {
        return Ok(None);
    }

    let mut packs = conn
        .prepare("SELECT sha256, path, byte_length, asset_count FROM artwork_pack ORDER BY sha256")
        .map_err(|error| catalog_error("读取旧 artwork_pack", error))?
        .query_map([], |row| {
            Ok(ArtworkPack {
                sha256: row.get(0)?,
                path: row.get(1)?,
                byte_length: row
                    .get::<_, i64>(2)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, i64::MAX))?,
                asset_count: row
                    .get::<_, i64>(3)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, i64::MAX))?,
            })
        })
        .map_err(|error| catalog_error("查询旧 artwork_pack", error))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| catalog_error("解析旧 artwork_pack", error))?;

    let assets = conn
        .prepare(
            "SELECT artwork_asset.sha256, artwork_pack.sha256, member_name, data_offset, \
             artwork_asset.byte_length, media_type, width, height \
             FROM artwork_asset INNER JOIN artwork_pack ON artwork_pack.id = artwork_asset.pack_id \
             ORDER BY artwork_asset.sha256",
        )
        .map_err(|error| catalog_error("读取旧 artwork_asset", error))?
        .query_map([], |row| {
            Ok(ArtworkAsset {
                sha256: row.get(0)?,
                pack_sha256: row.get(1)?,
                member_name: row.get(2)?,
                data_offset: row
                    .get::<_, i64>(3)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, i64::MAX))?,
                byte_length: row
                    .get::<_, i64>(4)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, i64::MAX))?,
                media_type: row.get(5)?,
                width: row
                    .get::<_, i64>(6)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(6, i64::MAX))?,
                height: row
                    .get::<_, i64>(7)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(7, i64::MAX))?,
            })
        })
        .map_err(|error| catalog_error("查询旧 artwork_asset", error))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| catalog_error("解析旧 artwork_asset", error))?;
    packs.sort_by(|left, right| left.sha256.cmp(&right.sha256));
    Ok(Some(PriorCatalog { packs, assets }))
}

fn tar_header(name: &str, size: u64) -> Result<[u8; 512]> {
    if name.len() > 100 || name.contains('/') || name.contains('\\') || !name.is_ascii() {
        return Err(AppError::LibraryIndexError(format!(
            "非法 tar member 名称: {name}"
        )));
    }
    let mut header = [0_u8; 512];
    header[..name.len()].copy_from_slice(name.as_bytes());
    write_octal(&mut header[100..108], 0o644)?;
    write_octal(&mut header[108..116], 0)?;
    write_octal(&mut header[116..124], 0)?;
    write_octal(&mut header[124..136], size)?;
    write_octal(&mut header[136..148], 0)?;
    header[148..156].fill(b' ');
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum = tar_checksum(&header);
    let checksum_text = format!("{checksum:06o}\0 ");
    header[148..156].copy_from_slice(checksum_text.as_bytes());
    Ok(header)
}

fn write_octal(field: &mut [u8], value: u64) -> Result<()> {
    let digits = field.len() - 1;
    let value = format!("{value:0digits$o}");
    if value.len() != digits {
        return Err(AppError::LibraryIndexError(
            "tar 数值超出字段范围".to_string(),
        ));
    }
    field[..digits].copy_from_slice(value.as_bytes());
    field[digits] = 0;
    Ok(())
}

fn tar_octal(field: &[u8]) -> Result<u64> {
    let value = field
        .iter()
        .copied()
        .take_while(|byte| *byte != 0 && *byte != b' ')
        .collect::<Vec<_>>();
    let text = std::str::from_utf8(&value)
        .map_err(|_| AppError::LibraryIndexError("tar 八进制字段不是 UTF-8".to_string()))?;
    u64::from_str_radix(text.trim(), 8)
        .map_err(|_| AppError::LibraryIndexError("tar 八进制字段无效".to_string()))
}

fn tar_string(field: &[u8]) -> String {
    String::from_utf8_lossy(field.split(|byte| *byte == 0).next().unwrap_or_default()).into_owned()
}

fn tar_checksum(header: &[u8; 512]) -> u64 {
    header
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            if (148..156).contains(&index) {
                u64::from(b' ')
            } else {
                u64::from(*byte)
            }
        })
        .sum()
}

fn align_tar(length: u64) -> u64 {
    length.saturating_add(TAR_BLOCK - 1) / TAR_BLOCK * TAR_BLOCK
}

fn write_zeros(writer: &mut File, mut length: u64) -> Result<()> {
    let zeros = [0_u8; 512];
    while length > 0 {
        let count = length.min(zeros.len() as u64) as usize;
        writer
            .write_all(&zeros[..count])
            .map_err(pack_io("写入 tar padding"))?;
        length -= count as u64;
    }
    Ok(())
}

fn temp_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "aniorg-{prefix}-{}-{}.{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed),
        extension
    ))
}

fn to_i64(value: u64, field: &str) -> Result<i64> {
    value
        .try_into()
        .map_err(|_| AppError::LibraryIndexError(format!("{field} 超出 SQLite INTEGER 范围")))
}

fn catalog_error(context: &'static str, error: rusqlite::Error) -> AppError {
    AppError::LibraryIndexError(format!("{context}失败: {error}"))
}

fn pack_io(context: &'static str) -> impl FnOnce(std::io::Error) -> AppError {
    move |error| AppError::LibraryIndexError(format!("{context}失败: {error}"))
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(File::open(path).map_err(pack_io("读取 SHA-256 文件"))?);
    sha256_reader(&mut reader)
}

fn sha256_reader(reader: &mut impl Read) -> Result<String> {
    let mut sha = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer).map_err(pack_io("计算 SHA-256"))?;
        if count == 0 {
            break;
        }
        sha.update(&buffer[..count]);
    }
    Ok(hex(&sha.finalize()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut sha = Sha256::new();
    sha.update(bytes);
    hex(&sha.finalize())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffered: usize,
    length: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09_e667,
                0xbb67_ae85,
                0x3c6e_f372,
                0xa54f_f53a,
                0x510e_527f,
                0x9b05_688c,
                0x1f83_d9ab,
                0x5be0_cd19,
            ],
            buffer: [0; 64],
            buffered: 0,
            length: 0,
        }
    }

    fn update(&mut self, mut bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        if self.buffered > 0 {
            let count = (64 - self.buffered).min(bytes.len());
            self.buffer[self.buffered..self.buffered + count].copy_from_slice(&bytes[..count]);
            self.buffered += count;
            bytes = &bytes[count..];
            if self.buffered == 64 {
                let block = self.buffer;
                self.compress(&block);
                self.buffered = 0;
            } else {
                return;
            }
        }
        while bytes.len() >= 64 {
            let block: &[u8; 64] = bytes[..64].try_into().expect("64-byte block");
            self.compress(block);
            bytes = &bytes[64..];
        }
        self.buffer[..bytes.len()].copy_from_slice(bytes);
        self.buffered = bytes.len();
    }

    fn finalize(mut self) -> [u8; 32] {
        let bit_length = self.length.wrapping_mul(8);
        let mut padding = [0_u8; 128];
        padding[0] = 0x80;
        let padding_length = if self.buffered < 56 {
            56 - self.buffered
        } else {
            120 - self.buffered
        };
        self.update(&padding[..padding_length]);
        self.update(&bit_length.to_be_bytes());
        let mut output = [0_u8; 32];
        for (chunk, value) in output.chunks_exact_mut(4).zip(self.state) {
            chunk.copy_from_slice(&value.to_be_bytes());
        }
        output
    }

    fn compress(&mut self, block: &[u8; 64]) {
        const K: [u32; 64] = [
            0x428a_2f98,
            0x7137_4491,
            0xb5c0_fbcf,
            0xe9b5_dba5,
            0x3956_c25b,
            0x59f1_11f1,
            0x923f_82a4,
            0xab1c_5ed5,
            0xd807_aa98,
            0x1283_5b01,
            0x2431_85be,
            0x550c_7dc3,
            0x72be_5d74,
            0x80de_b1fe,
            0x9bdc_06a7,
            0xc19b_f174,
            0xe49b_69c1,
            0xefbe_4786,
            0x0fc1_9dc6,
            0x240c_a1cc,
            0x2de9_2c6f,
            0x4a74_84aa,
            0x5cb0_a9dc,
            0x76f9_88da,
            0x983e_5152,
            0xa831_c66d,
            0xb003_27c8,
            0xbf59_7fc7,
            0xc6e0_0bf3,
            0xd5a7_9147,
            0x06ca_6351,
            0x1429_2967,
            0x27b7_0a85,
            0x2e1b_2138,
            0x4d2c_6dfc,
            0x5338_0d13,
            0x650a_7354,
            0x766a_0abb,
            0x81c2_c92e,
            0x9272_2c85,
            0xa2bf_e8a1,
            0xa81a_664b,
            0xc24b_8b70,
            0xc76c_51a3,
            0xd192_e819,
            0xd699_0624,
            0xf40e_3585,
            0x106a_a070,
            0x19a4_c116,
            0x1e37_6c08,
            0x2748_774c,
            0x34b0_bcb5,
            0x391c_0cb3,
            0x4ed8_aa4a,
            0x5b9c_ca4f,
            0x682e_6ff3,
            0x748f_82ee,
            0x78a5_636f,
            0x84c8_7814,
            0x8cc7_0208,
            0x90be_fffa,
            0xa450_6ceb,
            0xbef9_a3f7,
            0xc671_78f2,
        ];
        let mut words = [0_u32; 64];
        for (word, chunk) in words.iter_mut().zip(block.chunks_exact(4)) {
            *word = u32::from_be_bytes(chunk.try_into().expect("4-byte word"));
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(sum1)
                .wrapping_add(choose)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (state, value) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *state = state.wrapping_add(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_standard_vectors_and_chunked_reads() {
        let vectors = [
            (
                b"".as_slice(),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc".as_slice(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_slice(),
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ];
        for (input, expected) in vectors {
            assert_eq!(sha256_hex(input), expected);
            assert_eq!(
                sha256_reader(&mut ChunkedReader::new(input, 7)).unwrap(),
                expected
            );
        }
        let million = vec![b'a'; 1_000_000];
        assert_eq!(
            sha256_hex(&million),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
        for length in [55, 56, 63, 64, 65, 127, 128] {
            let input = vec![b'x'; length];
            assert_eq!(
                sha256_reader(&mut ChunkedReader::new(&input, 3)).unwrap(),
                sha256_hex(&input),
                "chunk boundary {length}"
            );
        }
    }

    struct ChunkedReader<'a> {
        input: &'a [u8],
        position: usize,
        chunk_size: usize,
    }

    impl<'a> ChunkedReader<'a> {
        fn new(input: &'a [u8], chunk_size: usize) -> Self {
            Self {
                input,
                position: 0,
                chunk_size,
            }
        }
    }

    impl Read for ChunkedReader<'_> {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            let remaining = &self.input[self.position..];
            let count = remaining.len().min(self.chunk_size).min(output.len());
            output[..count].copy_from_slice(&remaining[..count]);
            self.position += count;
            Ok(count)
        }
    }

    #[test]
    fn tar_header_uses_standard_checksum_and_size() {
        let header = tar_header(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad.png",
            123,
        )
        .unwrap();
        assert_eq!(tar_octal(&header[124..136]).unwrap(), 123);
        assert_eq!(tar_octal(&header[148..156]).unwrap(), tar_checksum(&header));
        assert_eq!(&header[257..265], b"ustar\x0000");
    }

    #[test]
    fn shards_roll_at_target_and_allow_one_oversize_asset() {
        let candidate = |sha256: &str, byte_length| ImageCandidate {
            source_path: PathBuf::new(),
            sha256: sha256.repeat(64),
            extension: "png".to_string(),
            media_type: "image/png".to_string(),
            width: 1,
            height: 1,
            byte_length,
        };
        let shards = shard_candidates(vec![
            candidate("a", 40 * 1024 * 1024),
            candidate("b", 40 * 1024 * 1024),
        ]);
        assert_eq!(shards.len(), 2);

        let oversize = shard_candidates(vec![candidate("c", MAX_PACK_BYTES + 1)]);
        assert_eq!(oversize.len(), 1);
        assert_eq!(oversize[0].len(), 1);
    }
}
