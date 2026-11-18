use std::collections::HashSet;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};
use trame::term::graphics::{Image, ImageTile, Layer, uv_to_source_rect};

use super::TermView;
use super::paint::Paint;

pub fn texture(view: &TermView, image: &Image) -> gdk::Texture {
    let mut textures = view.imp().textures.borrow_mut();
    textures
        .entry(image.serial)
        .or_insert_with(|| {
            let bytes = glib::Bytes::from_owned(image.pixels.clone());
            gdk::MemoryTexture::new(
                image.width as i32,
                image.height as i32,
                gdk::MemoryFormat::R8g8b8a8,
                &bytes,
                image.width as usize * 4,
            )
            .into()
        })
        .clone()
}

pub fn prune(view: &TermView, tiles: &[ImageTile]) {
    let live: HashSet<u64> = tiles.iter().map(|tile| tile.image.serial).collect();
    view.imp()
        .textures
        .borrow_mut()
        .retain(|serial, _| live.contains(serial));
}

pub fn draw_tiles(paint: &Paint, tiles: &[ImageTile], layer: Layer) {
    for tile in tiles.iter().filter(|tile| tile.layer == layer) {
        draw_tile(paint, tile);
    }
}

fn draw_tile(paint: &Paint, tile: &ImageTile) {
    let dst = tile_rect(paint, tile);
    let bounds = (dst.x(), dst.y(), dst.width(), dst.height());
    let Some((x, y, width, height)) = uv_to_source_rect(bounds, tile.uv) else {
        return;
    };
    let texture = texture(paint.view, &tile.image);
    let full = graphene::Rect::new(x, y, width, height);
    paint.snapshot.push_clip(&dst);
    paint
        .snapshot
        .append_scaled_texture(&texture, gsk::ScalingFilter::Linear, &full);
    paint.snapshot.pop();
}

fn tile_rect(paint: &Paint, tile: &ImageTile) -> graphene::Rect {
    let (width, height) = (paint.metrics.width, paint.metrics.height);
    graphene::Rect::new(
        tile.col as f32 * width + f32::from(tile.offset.0),
        tile.row as f32 * height + f32::from(tile.offset.1),
        tile.cols * width,
        tile.rows * height,
    )
}
