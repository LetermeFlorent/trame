use gtk::prelude::*;
use gtk::{gdk, gsk};
use trame::geometry::Rect;
use trame::term::{Screen, StoredImage};
use trame::ui::terminal::preview_rect;

use super::Paint;
use super::strips::{RowCells, image_strips, strips_bounds, to_graphene};
use crate::gnome::picture::rules::MAX_PREVIEW_SIDE;
use crate::gnome::view::images::{self, InlineTexture};
use crate::gnome::view::theme::{fill, rgba};
use crate::gnome::worker::PreviewKey;

const BORDER_WIDTH: f32 = 1.0;
const FLAT_RADIUS: f32 = 0.0;

struct TileHit<'a> {
    rows: &'a [RowCells<'a>],
    row: &'a RowCells<'a>,
    col: usize,
    id: u32,
}

pub fn draw_images(paint: &Paint, screen: &Screen, rows: &[RowCells]) {
    let mut seen = Vec::new();
    for row in rows {
        for (col, cell) in row.cells.iter().enumerate() {
            let Some(id) = cell.image_id() else {
                continue;
            };
            if seen.contains(&id) {
                continue;
            }
            seen.push(id);
            let tile = TileHit { rows, row, col, id };
            draw_preview(paint, screen, &tile);
        }
    }
}

fn draw_preview(paint: &Paint, screen: &Screen, tile: &TileHit) {
    let cell_size = (paint.metrics.width, paint.metrics.height);
    let strips = image_strips(tile.rows, tile.id, cell_size);
    let Some(stored) = screen.image(tile.id) else {
        draw_broken(paint, strips_bounds(&strips), &strips);
        return;
    };
    let Some(tile_at) = tile.row.cells[tile.col].image_tile() else {
        return;
    };
    let box_rect = image_box(tile, tile_at, stored, cell_size);
    let target = preview_rect(box_rect, stored.width, stored.height);
    let limit = (
        (box_rect.width.round().max(1.0) as u32).min(MAX_PREVIEW_SIDE),
        (box_rect.height.round().max(1.0) as u32).min(MAX_PREVIEW_SIDE),
    );
    let key = PreviewKey {
        session_id: paint.session_id,
        image_id: tile.id,
        width: target.width.round() as u32,
        height: target.height.round() as u32,
    };
    match images::lookup_inline(paint.view, key, limit, &stored.bytes) {
        InlineTexture::Ready(texture) => draw_strips(paint, &texture, target, &strips),
        InlineTexture::Broken => draw_broken(paint, box_rect, &strips),
        InlineTexture::Pending => {}
    }
}

fn image_box(
    tile: &TileHit,
    tile_at: (u16, u16),
    stored: &StoredImage,
    cell_size: (f32, f32),
) -> Rect {
    Rect::new(
        (tile.col as f32 - f32::from(tile_at.1)) * cell_size.0,
        tile.row.y - f32::from(tile_at.0) * cell_size.1,
        f32::from(stored.cols) * cell_size.0,
        f32::from(stored.rows) * cell_size.1,
    )
}

fn draw_strips(paint: &Paint, texture: &gdk::MemoryTexture, target: Rect, strips: &[Rect]) {
    let placed = to_graphene(target);
    for &strip in strips {
        let clip = paint.bounds.intersect(strip);
        if clip.is_empty() {
            continue;
        }
        paint.snapshot.push_clip(&to_graphene(clip));
        paint.snapshot.append_texture(texture, &placed);
        paint.snapshot.pop();
    }
}

fn draw_broken(paint: &Paint, rect: Rect, strips: &[Rect]) {
    let area = to_graphene(rect);
    let outline = gsk::RoundedRect::from_rect(area, FLAT_RADIUS);
    let border = rgba(paint.theme.border_strong);
    let widths = [BORDER_WIDTH; 4];
    let colors = [border; 4];
    for &strip in strips {
        let clip = paint.bounds.intersect(strip);
        if clip.is_empty() {
            continue;
        }
        paint.snapshot.push_clip(&to_graphene(clip));
        fill(paint.snapshot, paint.theme.hover, area);
        paint.snapshot.append_border(&outline, &widths, &colors);
        paint.snapshot.pop();
    }
}
