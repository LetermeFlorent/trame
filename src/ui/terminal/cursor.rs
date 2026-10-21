use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::{CursorShape, Flags, Screen};
use crate::text::CellMetrics;
use crate::theme::Theme;

const BAR_RATIO: f32 = 1.0 / 7.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorPlace {
    pub row: usize,
    pub col: usize,
    pub columns: usize,
    pub shape: CursorShape,
}

impl CursorPlace {
    pub fn locate(screen: &Screen) -> Option<Self> {
        let view = screen.cursor();
        let grid = screen.grid();
        let row = view.row + grid.display_offset();
        if !view.visible || row >= grid.rows() || grid.cols() == 0 {
            return None;
        }
        let mut col = view.col.min(grid.cols() - 1);
        let line = grid.visible_row(row);
        if col > 0 && line.cell(col).flags.contains(Flags::WIDE_SPACER) {
            col -= 1;
        }
        let columns = if line.cell(col).flags.contains(Flags::WIDE) {
            2
        } else {
            1
        };
        Some(Self {
            row,
            col,
            columns,
            shape: view.shape,
        })
    }

    pub fn covers(&self, row: usize, col: usize) -> bool {
        self.row == row && (self.col..self.col + self.columns).contains(&col)
    }

    pub fn fills_cell(&self, focused: bool) -> bool {
        focused && self.shape == CursorShape::Block
    }

    pub fn rect(&self, origin: Point, metrics: &CellMetrics) -> Rect {
        Rect::new(
            origin.x + self.col as f32 * metrics.width,
            origin.y + self.row as f32 * metrics.height,
            metrics.width * self.columns as f32,
            metrics.height,
        )
    }

    pub fn draw_fill(&self, scene: &mut Scene, theme: &Theme, cell: Rect) {
        scene.fill(cell, theme.cursor, 0.0);
    }

    pub fn draw_outline(
        &self,
        scene: &mut Scene,
        theme: &Theme,
        cell: Rect,
        metrics: &CellMetrics,
        focused: bool,
    ) {
        let stroke = (metrics.font_size * BAR_RATIO).round().max(1.0);
        match self.shape {
            CursorShape::Block if !focused => {
                scene.border(cell, theme.cursor, 0.0, (stroke / 2.0).round().max(1.0));
            }
            CursorShape::Block => {}
            CursorShape::Bar => {
                scene.fill(
                    Rect::new(cell.x, cell.y, stroke, cell.height),
                    theme.cursor,
                    0.0,
                );
            }
            CursorShape::Underline => {
                scene.fill(
                    Rect::new(cell.x, cell.bottom() - stroke, cell.width, stroke),
                    theme.cursor,
                    0.0,
                );
            }
        }
    }
}
