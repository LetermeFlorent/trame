use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Image {
    pub serial: u64,
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<[u8]>,
}
