use std::path::PathBuf;
use std::sync::Arc;

use gtk::subclass::prelude::*;
use trame::files::open_link;
use trame::geometry::Rect;
use trame::text::ImageSource;
use trame::ui::terminal::links::{LinkHit, LinkKind, link_hit};

use crate::gnome::picture::rules::{
    grid_cell, inline_click_opens, link_active, reports_mouse, span_anchor,
};
use crate::gnome::view::{TermView, live, metrics};

enum HitOutcome {
    Inline(Arc<[u8]>),
    ImageLink(PathBuf),
    WebLink(String),
}

pub fn activate_press(view: &TermView) -> bool {
    if view.picture_open() {
        return true;
    }
    let Some(outcome) = hit_at_pointer(view) else {
        return false;
    };
    match outcome {
        HitOutcome::Inline(bytes) => trigger_picture(view, ImageSource::Bytes(bytes), None),
        HitOutcome::ImageLink(path) => {
            trigger_picture(view, ImageSource::from(path.clone()), Some(path));
        }
        HitOutcome::WebLink(url) => open_link(url),
    }
    true
}

fn trigger_picture(view: &TermView, source: ImageSource, path: Option<PathBuf>) {
    if let Some(open) = view.imp().picture.borrow().as_ref() {
        open(source, path);
    }
}

fn hit_at_pointer(view: &TermView) -> Option<HitOutcome> {
    let cur = view.imp().pointer.get();
    let point = cur.position?;
    let cwd = live::session_cwd(view);
    let cell_metrics = metrics::current(view);
    let session = view.imp().session.borrow();
    let terminal = session.as_ref()?.terminal();
    let screen = terminal.screen();
    let cell = grid_cell(
        point,
        (cell_metrics.width, cell_metrics.height),
        (screen.rows(), screen.cols()),
    )?;
    let reports = reports_mouse(screen.modes(), cur.shift);
    if inline_click_opens(reports, cur.ctrl)
        && let Some((_, stored)) = screen.image_at(cell.0, cell.1)
    {
        return Some(HitOutcome::Inline(Arc::clone(&stored.bytes)));
    }
    if link_active(false, reports, cur.ctrl)
        && let Some(hit) = link_hit(screen, cell.0 as isize, cell.1, cwd.as_deref())
    {
        return Some(match hit.kind {
            LinkKind::Image => HitOutcome::ImageLink(PathBuf::from(hit.target)),
            LinkKind::Web => HitOutcome::WebLink(hit.target),
        });
    }
    None
}

pub fn inspect_hover(
    view: &TermView,
    point: (f32, f32),
    ctrl: bool,
    shift: bool,
) -> (bool, Option<(PathBuf, Rect)>) {
    let cwd = live::session_cwd(view);
    let cell_metrics = metrics::current(view);
    let cell_size = (cell_metrics.width, cell_metrics.height);
    let session = view.imp().session.borrow();
    let Some(session) = session.as_ref() else {
        return (false, None);
    };
    let terminal = session.terminal();
    let screen = terminal.screen();
    let Some(cell) = grid_cell(point, cell_size, (screen.rows(), screen.cols())) else {
        return (false, None);
    };
    let reports = reports_mouse(screen.modes(), shift);
    let inline = inline_click_opens(reports, ctrl) && screen.image_at(cell.0, cell.1).is_some();
    let hit = link_active(false, reports, ctrl)
        .then(|| link_hit(screen, cell.0 as isize, cell.1, cwd.as_deref()))
        .flatten();
    let clickable = inline || hit.is_some();
    let preview = hit.and_then(|found| image_hover_target(found, cell, cell_size));
    (clickable, preview)
}

fn image_hover_target(
    hit: LinkHit,
    cell: (usize, usize),
    cell_size: (f32, f32),
) -> Option<(PathBuf, Rect)> {
    if hit.kind != LinkKind::Image {
        return None;
    }
    let row = cell.0 as isize;
    let span = hit
        .spans
        .iter()
        .find(|item| item.row == row && item.covers(cell.1))
        .or_else(|| hit.spans.first())?;
    Some((PathBuf::from(hit.target), span_anchor(span, cell_size)))
}
