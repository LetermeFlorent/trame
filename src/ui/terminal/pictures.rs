use super::row::preview_fit;
use super::strips::{image_strips, strips_bounds};
use super::{GridPaint, RowPaint};
use crate::geometry::Rect;
use crate::render::{AtlasRegion, Scene};
use crate::term::StoredImage;
use crate::text::{ImageKey, TextSystem};
use crate::theme::Rgba;

struct TileHit<'a> {
    rows: &'a [RowPaint<'a>],
    row: &'a RowPaint<'a>,
    col: usize,
    id: u32,
    clip: Rect,
}

pub(super) fn draw_images(
    scene: &mut Scene,
    text: &mut TextSystem,
    grid: &GridPaint,
    rows: &[RowPaint],
) {
    let Some((first, last)) = rows.first().zip(rows.last()) else {
        return;
    };
    let clip = Rect::new(
        grid.paint.origin.x,
        first.y,
        grid.paint.screen.cols as f32 * grid.metrics.width,
        last.y + grid.metrics.height - first.y,
    );
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
            let tile = TileHit {
                rows,
                row,
                col,
                id,
                clip,
            };
            draw_preview(scene, text, grid, &tile);
        }
    }
}

fn draw_preview(scene: &mut Scene, text: &mut TextSystem, grid: &GridPaint, tile: &TileHit) {
    let strips = image_strips(tile.rows, tile.id, grid.paint.origin.x, &grid.metrics);
    let Some(stored) = grid.paint.screen.image(tile.id) else {
        draw_broken(scene, grid, strips_bounds(&strips), &strips, tile.clip);
        return;
    };
    let Some(tile_at) = tile.row.cells[tile.col].image_tile() else {
        return;
    };
    let box_rect = image_box(grid, tile, tile_at, stored);
    let (target, max_w, max_h) =
        preview_fit(box_rect, stored.width, stored.height, text.max_image_side());
    let key = ImageKey {
        id: (grid.paint.screen.session_id() << 24) | u64::from(tile.id),
        width: target.width.round() as u32,
        height: target.height.round() as u32,
    };
    match text.queued_image(key, (max_w, max_h), &stored.bytes) {
        Some(region) => draw_strips(scene, region, target, &strips, tile.clip),
        None => draw_broken(scene, grid, box_rect, &strips, tile.clip),
    }
}

fn image_box(grid: &GridPaint, tile: &TileHit, tile_at: (u16, u16), stored: &StoredImage) -> Rect {
    let metrics = &grid.metrics;
    Rect::new(
        grid.paint.origin.x + (tile.col as f32 - f32::from(tile_at.1)) * metrics.width,
        tile.row.y - f32::from(tile_at.0) * metrics.height,
        f32::from(stored.cols) * metrics.width,
        f32::from(stored.rows) * metrics.height,
    )
}

fn draw_strips(scene: &mut Scene, region: AtlasRegion, target: Rect, strips: &[Rect], clip: Rect) {
    for &strip in strips {
        scene.push_clip(clip.intersect(strip));
        scene.glyph(region, target.x, target.y, Rgba::WHITE);
        scene.pop_clip();
    }
}

fn draw_broken(scene: &mut Scene, grid: &GridPaint, rect: Rect, strips: &[Rect], clip: Rect) {
    let light = grid.metrics.underline_thickness.round().max(1.0);
    for &strip in strips {
        scene.push_clip(clip.intersect(strip));
        scene.fill(rect, grid.theme.hover, 0.0);
        scene.border(rect, grid.theme.border_strong, 0.0, light);
        scene.pop_clip();
    }
}
