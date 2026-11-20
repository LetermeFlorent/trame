use gtk::graphene;
use trame::geometry::Rect;
use trame::term::Cell;

pub struct RowCells<'a> {
    pub row: isize,
    pub y: f32,
    pub cells: &'a [Cell],
}

pub fn image_strips(rows: &[RowCells], id: u32, cell_size: (f32, f32)) -> Vec<Rect> {
    let mut strips: Vec<Rect> = Vec::new();
    let mut prev_row: Option<isize> = None;
    let mut prev_start = 0usize;
    for row in rows {
        let row_start = strips.len();
        push_row_strips(&mut strips, row, id, cell_size);
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

fn push_row_strips(strips: &mut Vec<Rect>, row: &RowCells, id: u32, cell_size: (f32, f32)) {
    let mut start: Option<usize> = None;
    for (col, cell) in row.cells.iter().enumerate() {
        match (start, cell.image_id() == Some(id)) {
            (None, true) => start = Some(col),
            (Some(first), false) => {
                strips.push(span_rect(row.y, first, col, cell_size));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(first) = start {
        strips.push(span_rect(row.y, first, row.cells.len(), cell_size));
    }
}

fn span_rect(y: f32, start: usize, end: usize, cell_size: (f32, f32)) -> Rect {
    Rect::new(
        start as f32 * cell_size.0,
        y,
        (end - start) as f32 * cell_size.0,
        cell_size.1,
    )
}

pub fn strips_bounds(strips: &[Rect]) -> Rect {
    strips
        .iter()
        .copied()
        .reduce(|acc, rect| acc.union(rect))
        .unwrap_or_default()
}

pub fn to_graphene(rect: Rect) -> graphene::Rect {
    graphene::Rect::new(rect.x, rect.y, rect.width, rect.height)
}
