use crate::ui::chrome::{Painter, Press};
use crate::ui::frame::ScrollBar;
use crate::ui::hit::Hit;

pub(in crate::ui::chrome) fn draw_bar(
    painter: &mut Painter,
    bar: ScrollBar,
    track: Hit,
    thumb: Hit,
) {
    let theme = painter.theme;
    let radius = bar.track.width / 2.0;
    if painter.state(track) != Press::Idle {
        painter.scene.fill(bar.track, theme.hover, radius);
    }
    let color = match painter.state(thumb) {
        Press::Idle => theme.border_strong,
        _ => theme.text_faint,
    };
    painter.scene.fill(bar.thumb, color, radius);
}
