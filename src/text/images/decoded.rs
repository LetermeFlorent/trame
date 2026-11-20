use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use std::time::SystemTime;

use super::source::ImageSource;
use crate::files::pictures::{
    MAX_IMAGE_FILE_BYTES, RgbaImage, decode, image_limits, read_image_bytes,
};
use crate::log_error;

const MAX_CACHE_ENTRIES: usize = 4;
const MAX_CACHE_BYTES: usize = 64 * 1024 * 1024;
const BYTES_PER_PIXEL: usize = 4;

enum CacheKey {
    Bytes(Weak<[u8]>),
    File {
        path: Arc<PathBuf>,
        len: u64,
        modified: Option<SystemTime>,
    },
}

struct CachedDecode {
    key: CacheKey,
    image: Arc<RgbaImage>,
    bytes: usize,
}

#[derive(Default)]
pub(super) struct DecodedCache {
    entries: VecDeque<CachedDecode>,
    total_bytes: usize,
}

impl DecodedCache {
    pub(super) fn get_or_decode(&mut self, source: &ImageSource) -> Option<Arc<RgbaImage>> {
        self.entries.retain(|entry| match &entry.key {
            CacheKey::Bytes(weak) => weak.strong_count() > 0,
            CacheKey::File { .. } => true,
        });
        self.total_bytes = self.entries.iter().map(|entry| entry.bytes).sum();
        match source {
            ImageSource::Bytes(raw) => self.get_bytes(raw),
            ImageSource::Path(path) => self.get_file(path),
        }
    }

    fn get_bytes(&mut self, raw: &Arc<[u8]>) -> Option<Arc<RgbaImage>> {
        let hit = self.entries.iter().find_map(|entry| match &entry.key {
            CacheKey::Bytes(weak) => {
                Arc::ptr_eq(&weak.upgrade()?, raw).then(|| Arc::clone(&entry.image))
            }
            CacheKey::File { .. } => None,
        });
        if hit.is_some() {
            return hit;
        }
        let decoded = decode_logged(raw)?;
        self.store(CacheKey::Bytes(Arc::downgrade(raw)), decoded)
    }

    fn get_file(&mut self, path: &Arc<PathBuf>) -> Option<Arc<RgbaImage>> {
        let meta = fs::metadata(path.as_ref())
            .inspect_err(|err| log_error!("the image preview failed: {err}"))
            .ok()?;
        if !meta.is_file() || meta.len() > MAX_IMAGE_FILE_BYTES {
            return None;
        }
        let (len, modified) = (meta.len(), meta.modified().ok());
        let hit = self.entries.iter().find_map(|entry| match &entry.key {
            CacheKey::File {
                path: p,
                len: l,
                modified: m,
            } if p.as_path() == path.as_path() && *l == len && *m == modified => {
                Some(Arc::clone(&entry.image))
            }
            _ => None,
        });
        if hit.is_some() {
            return hit;
        }
        self.entries.retain(
            |e| !matches!(&e.key, CacheKey::File { path: p, .. } if p.as_path() == path.as_path()),
        );
        self.total_bytes = self.entries.iter().map(|e| e.bytes).sum();
        let bytes = read_image_bytes(path)
            .inspect_err(|err| log_error!("the image preview failed: {err}"))
            .ok()?;
        let decoded = decode_logged(&bytes)?;
        let key = CacheKey::File {
            path: Arc::clone(path),
            len,
            modified,
        };
        self.store(key, decoded)
    }

    fn store(&mut self, key: CacheKey, decoded: RgbaImage) -> Option<Arc<RgbaImage>> {
        let bytes = (decoded.width() as usize)
            .saturating_mul(decoded.height() as usize)
            .saturating_mul(BYTES_PER_PIXEL);
        let image = Arc::new(decoded);
        if bytes <= MAX_CACHE_BYTES {
            while self.entries.len() >= MAX_CACHE_ENTRIES
                || self.total_bytes + bytes > MAX_CACHE_BYTES
            {
                let Some(oldest) = self.entries.pop_front() else {
                    break;
                };
                self.total_bytes = self.total_bytes.saturating_sub(oldest.bytes);
            }
            self.total_bytes += bytes;
            self.entries.push_back(CachedDecode {
                key,
                image: Arc::clone(&image),
                bytes,
            });
        }
        Some(image)
    }
}

fn decode_logged(raw: &[u8]) -> Option<RgbaImage> {
    decode(raw, image_limits())
        .inspect_err(|err| log_error!("the image preview failed: {err}"))
        .ok()
}
