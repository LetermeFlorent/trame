use super::GridPaint;
use super::row::RowPaint;
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::graphics::{ImageTile, Layer};
use crate::text::CellMetrics;

const SELECTION_OVER_IMAGE: f32 = 0.5;

pub fn draw_tiles(
    scene: &mut Scene,
    metrics: &CellMetrics,
    origin: Point,
    tiles: &[ImageTile],
    layer: Layer,
) {
    for tile in tiles.iter().filter(|tile| tile.layer == layer) {
        scene.image(&tile.image, tile_rect(metrics, origin, tile), tile.uv);
    }
}

pub fn tile_rect(metrics: &CellMetrics, origin: Point, tile: &ImageTile) -> Rect {
    Rect::new(
        origin.x + tile.col as f32 * metrics.width + f32::from(tile.offset.0),
        origin.y + tile.row as f32 * metrics.height + f32::from(tile.offset.1),
        tile.cols * metrics.width,
        tile.rows * metrics.height,
    )
}

pub(super) fn cover_tiles(scene: &mut Scene, grid: &GridPaint, rows: &[RowPaint], origin: Point) {
    let tiles = &grid.paint.screen.tiles;
    let rects: Vec<(Layer, Rect)> = tiles
        .iter()
        .map(|tile| (tile.layer, tile_rect(&grid.metrics, origin, tile)))
        .collect();
    let covered = |cell: Rect, over: bool| {
        rects.iter().any(|(layer, rect)| {
            (!over || *layer == Layer::OverText) && !rect.intersect(cell).is_empty()
        })
    };
    let selection = grid.theme.selection.faded(SELECTION_OVER_IMAGE);
    for row in rows {
        for col in 0..grid.paint.screen.cols {
            let cell = cell_rect(&grid.metrics, origin, row, col);
            if (grid.paint.selected)(row.row, col) && covered(cell, false) {
                scene.fill(cell, selection, 0.0);
            }
        }
    }
    let cursor = grid
        .cursor
        .filter(|cursor| cursor.fills_cell(grid.paint.focused));
    if let Some(cursor) = cursor {
        let cell = cursor.rect(origin, &grid.metrics);
        if covered(cell, true) {
            cursor.draw_fill(scene, grid.theme, cell);
        }
    }
}

fn cell_rect(metrics: &CellMetrics, origin: Point, row: &RowPaint, col: usize) -> Rect {
    Rect::new(
        origin.x + col as f32 * metrics.width,
        row.y,
        metrics.width,
        metrics.height,
    )
}
