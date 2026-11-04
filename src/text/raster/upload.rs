use swash::scale::Source;
use swash::scale::image::{Content, Image};

use super::RasterGlyph;
use crate::render::{Atlas, AtlasKind};

const BYTES_PER_PIXEL: usize = 4;
const ALPHA: usize = BYTES_PER_PIXEL - 1;
const OPAQUE: u32 = u8::MAX as u32;

pub(super) fn upload(atlas: &mut Atlas, image: Image) -> Option<RasterGlyph> {
    let kind = match image.content {
        Content::Color => AtlasKind::Color,
        Content::Mask | Content::SubpixelMask => AtlasKind::Mask,
    };
    let data = match image.source {
        Source::ColorOutline(_) => unpremultiply(image.data),
        _ => image.data,
    };
    let region = atlas.insert(kind, image.placement.width, image.placement.height, data)?;
    Some(RasterGlyph {
        region,
        left: image.placement.left,
        top: image.placement.top,
    })
}

fn unpremultiply(mut data: Vec<u8>) -> Vec<u8> {
    for pixel in data.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
        let alpha = u32::from(pixel[ALPHA]);
        if alpha == 0 || alpha == OPAQUE {
            continue;
        }
        for channel in &mut pixel[..ALPHA] {
            *channel = ((u32::from(*channel) * OPAQUE + alpha / 2) / alpha).min(OPAQUE) as u8;
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiplied_colors_are_restored() {
        let restored = unpremultiply(vec![64, 32, 0, 128, 10, 20, 30, 0, 1, 2, 3, 255]);
        assert_eq!(restored, vec![128, 64, 0, 128, 10, 20, 30, 0, 1, 2, 3, 255]);
    }
}
