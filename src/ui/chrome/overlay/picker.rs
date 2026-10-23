use crate::geometry::{Point, Rect};
use crate::theme::{Hsv, Rgba};
use crate::ui::chrome::Painter;
use crate::ui::frame::PickerFrame;
use crate::ui::metrics::UiMetrics;

use super::editor::draw_field;

const STRIP: f32 = 2.0;
const THUMB: f32 = 10.0;
const MARK: f32 = 4.0;
const OUTLINE_ALPHA: f32 = 0.5;
const WHITE: Rgba = Rgba::hex(0xFFFFFF);
const BLACK: Rgba = Rgba::hex(0x000000);

pub(super) fn draw_picker(painter: &mut Painter, picker: &PickerFrame) {
    draw_square(painter, picker.square, picker.hsv);
    draw_hue(painter, picker.hue);
    draw_marks(painter, picker);
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let line = painter.metrics.line();
    let color = Rgba::hex(picker.hsv.to_rgb());
    painter.scene.fill(picker.preview, color, radius);
    painter
        .scene
        .border(picker.preview, theme.border_strong, radius, line);
    draw_field(painter, &picker.field, true);
}

fn bands(from: f32, length: f32, step: f32) -> impl Iterator<Item = (f32, f32, f32)> {
    let count = (length / step.max(1.0)).ceil().max(1.0) as usize;
    let edge = move |at: usize| (from + length * at as f32 / count as f32).round();
    (0..count).map(move |index| {
        let share = (index as f32 + 0.5) / count as f32;
        (edge(index), edge(index + 1), share)
    })
}

fn draw_square(painter: &mut Painter, square: Rect, hsv: Hsv) {
    let step = painter.metrics.px(STRIP);
    let scene = &mut *painter.scene;
    scene.fill(square, Rgba::hex(hsv.pure_hue()), 0.0);
    for (left, right, share) in bands(square.x, square.width, step) {
        let column = Rect::from_edges(left, square.y, right, square.bottom());
        scene.fill(column, WHITE.with_alpha(1.0 - share), 0.0);
    }
    for (top, bottom, share) in bands(square.y, square.height, step) {
        let row = Rect::from_edges(square.x, top, square.right(), bottom);
        scene.fill(row, BLACK.with_alpha(share), 0.0);
    }
    let border = painter.theme.border_strong;
    painter
        .scene
        .border(square, border, 0.0, painter.metrics.line());
}

fn draw_hue(painter: &mut Painter, bar: Rect) {
    let step = painter.metrics.px(STRIP);
    for (top, bottom, hue) in bands(bar.y, bar.height, step) {
        let strip = Rect::from_edges(bar.x, top, bar.right(), bottom);
        let color = Hsv {
            hue,
            saturation: 1.0,
            value: 1.0,
        };
        painter.scene.fill(strip, Rgba::hex(color.to_rgb()), 0.0);
    }
    let border = painter.theme.border_strong;
    painter
        .scene
        .border(bar, border, 0.0, painter.metrics.line());
}

fn draw_marks(painter: &mut Painter, picker: &PickerFrame) {
    let line = painter.metrics.line();
    let Hsv {
        hue,
        saturation,
        value,
    } = picker.hsv;
    let (square, bar) = (picker.square, picker.hue);
    let size = painter.metrics.px(THUMB);
    let center = Point::new(
        square.x + saturation * square.width,
        square.y + (1.0 - value) * square.height,
    );
    let thumb = Rect::new(center.x - size / 2.0, center.y - size / 2.0, size, size).snapped();
    outlined(painter, thumb, size / 2.0, line);
    let height = painter.metrics.px(MARK);
    let y = (bar.y + hue * bar.height - height / 2.0).round();
    let mark = Rect::new(bar.x - line, y, bar.width + 2.0 * line, height);
    outlined(painter, mark, line, line);
}

fn outlined(painter: &mut Painter, rect: Rect, radius: f32, line: f32) {
    let scene = &mut *painter.scene;
    scene.border(rect, WHITE, radius, 2.0 * line);
    scene.border(
        rect.inset(-line),
        BLACK.with_alpha(OUTLINE_ALPHA),
        radius + line,
        line,
    );
}
