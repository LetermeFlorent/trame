use std::path::PathBuf;
use std::sync::Arc;

use super::ImageKey;

#[derive(Clone, Debug)]
pub enum ImageSource {
    Bytes(Arc<[u8]>),
    Path(Arc<PathBuf>),
}

impl From<Arc<[u8]>> for ImageSource {
    fn from(bytes: Arc<[u8]>) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<PathBuf> for ImageSource {
    fn from(path: PathBuf) -> Self {
        Self::Path(Arc::new(path))
    }
}

impl From<Arc<PathBuf>> for ImageSource {
    fn from(path: Arc<PathBuf>) -> Self {
        Self::Path(path)
    }
}

#[derive(Debug)]
pub struct DecodedPicture {
    pub width: u32,
    pub height: u32,
    pub src_size: (u32, u32),
    pub pixels: Vec<u8>,
}

#[derive(Debug)]
pub(super) struct PictureAnswer {
    pub(super) job: u64,
    pub(super) image: Option<DecodedPicture>,
}

pub(super) struct InlineJob {
    pub(super) key: ImageKey,
    pub(super) limit: (u32, u32),
    pub(super) bytes: Arc<[u8]>,
}

pub(super) struct InlineAnswer {
    pub(super) key: ImageKey,
    pub(super) pixels: Option<Vec<u8>>,
}

pub(super) struct PictureJob {
    pub(super) job: u64,
    pub(super) limit: (u32, u32),
    pub(super) source: ImageSource,
}

#[derive(Clone, Copy)]
pub(super) enum SlotTarget {
    Picture,
    Preview,
}
