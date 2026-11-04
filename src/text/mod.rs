mod cells;
mod font;
mod images;
mod layout;
mod metrics;
mod raster;
mod system;

use std::collections::HashMap;
use std::hash::Hash;

pub use cells::CellText;
pub use font::{BOLD, REGULAR, SEMIBOLD};
pub use images::ImageKey;
pub use layout::{FontRole, TextLayout, TextStyle};
pub use metrics::{CellMetrics, DEFAULT_LINE_HEIGHT};
pub use system::TextSystem;

const MAX_CACHED_GLYPHS: usize = 8192;

fn insert_bounded<K: Eq + Hash, V>(cache: &mut HashMap<K, V>, key: K, value: V) {
    if cache.len() >= MAX_CACHED_GLYPHS {
        cache.clear();
    }
    cache.insert(key, value);
}
