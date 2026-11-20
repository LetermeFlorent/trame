use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::geometry::Rect;
use trame::theme::Theme;
use trame::ui::UiMetrics;
use trame::ui::chrome::place_hover_preview;

use super::strips::to_graphene;
use crate::gnome::view::TermView;
use crate::gnome::view::theme::{fill, rgba};

const TITLE_TOP: f32 = 0.0;
const SHADOW_DX: f32 = 0.0;
const SHADOW_DY: f32 = 2.0;
const SHADOW_SPREAD: f32 = 0.0;
const SHADOW_BLUR: f32 = 6.0;

pub fn draw_card(view: &TermView, snapshot: &gtk::Snapshot, theme: &Theme, bounds: Rect) {
    if view.picture_open() {
        return;
    }
    let hover = view.imp().hover.borrow();
    let Some(active) = hover.active.as_ref() else {
        return;
    };
    let Some((width, height, ref texture)) = active.ready else {
        return;
    };
    let scale = (view.scale_factor().max(1)) as f32;
    let metrics = UiMetrics::new(scale);
    let (card, image) =
        place_hover_preview(bounds, TITLE_TOP, active.anchor, &metrics, (width, height));
    let card_rect = to_graphene(card);
    let radius = metrics.px(UiMetrics::RADIUS);
    let line = metrics.line();
    let blur = metrics.px(SHADOW_BLUR);
    let rounded = gsk::RoundedRect::from_rect(card_rect, radius);
    snapshot.append_outset_shadow(
        &rounded,
        &rgba(theme.shadow),
        SHADOW_DX,
        SHADOW_DY,
        SHADOW_SPREAD,
        blur,
    );
    snapshot.push_rounded_clip(&rounded);
    fill(snapshot, theme.chrome, card_rect);
    snapshot.pop();
    let border = rgba(theme.border_strong);
    snapshot.append_border(&rounded, &[line; 4], &[border; 4]);
    snapshot.append_texture(texture, &to_graphene(image));
}
