use super::RowPaint;
use crate::geometry::Rect;
use crate::text::CellMetrics;

pub(super) fn image_strips(
    rows: &[RowPaint],
    id: u32,
    origin_x: f32,
    metrics: &CellMetrics,
) -> Vec<Rect> {
    let mut strips: Vec<Rect> = Vec::new();
    let mut prev_row: Option<isize> = None;
    let mut prev_start = 0usize;
    for row in rows {
        let row_start = strips.len();
        push_row_strips(&mut strips, row, id, origin_x, metrics);
        if strips.len() == row_start + 1
            && prev_row == Some(row.row - 1)
            && row_start - prev_start == 1
        {
            let last = strips[row_start];
            let prev = &mut strips[prev_start];
            if prev.x == last.x && prev.width == last.width {
                prev.height = last.bottom() - prev.y;
                strips.pop();
                prev_row = Some(row.row);
                continue;
            }
        }
        prev_row = (strips.len() > row_start).then_some(row.row);
        prev_start = row_start;
    }
    strips
}

fn push_row_strips(
    strips: &mut Vec<Rect>,
    row: &RowPaint,
    id: u32,
    origin_x: f32,
    metrics: &CellMetrics,
) {
    let mut start: Option<usize> = None;
    for (col, cell) in row.cells.iter().enumerate() {
        match (start, cell.image_id() == Some(id)) {
            (None, true) => start = Some(col),
            (Some(first), false) => {
                strips.push(span_rect(origin_x, row.y, first, col, metrics));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(first) = start {
        strips.push(span_rect(origin_x, row.y, first, row.cells.len(), metrics));
    }
}

fn span_rect(origin_x: f32, y: f32, start: usize, end: usize, metrics: &CellMetrics) -> Rect {
    Rect::new(
        origin_x + start as f32 * metrics.width,
        y,
        (end - start) as f32 * metrics.width,
        metrics.height,
    )
}

pub(super) fn strips_bounds(strips: &[Rect]) -> Rect {
    strips
        .iter()
        .copied()
        .reduce(|acc, rect| acc.union(rect))
        .unwrap_or_default()
}
