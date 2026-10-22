mod row;

use std::sync::Arc;

use super::{FrameLayout, SidebarItem};
use crate::geometry::{Point, Rect};
use crate::input::TextField;
use crate::session::SessionId;
use crate::text::{TextLayout, TextSystem};
use crate::theme::Swatch;
use crate::ui::metrics::UiMetrics;
use crate::workspace::{InstanceId, PaneId};
pub use row::SwatchSlot;
use row::swatch_row;

const FIELD_INSET_Y: f32 = 2.0;
const TEXT_PAD: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditTarget {
    Instance(InstanceId),
    Tab(SessionId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditAnchor {
    Strip(PaneId),
    Sidebar,
}

pub struct EditorRequest<'a> {
    pub target: EditTarget,
    pub anchor: EditAnchor,
    pub field: &'a TextField,
    pub color: Option<Swatch>,
}

#[derive(Clone, Debug)]
pub struct EditorFrame {
    pub target: EditTarget,
    pub field: Rect,
    pub text_area: Rect,
    pub label: Arc<TextLayout>,
    pub text_x: f32,
    pub caret: Rect,
    pub strip: Rect,
    pub slots: Vec<SwatchSlot>,
    pub selected: Option<Swatch>,
}

impl EditorFrame {
    pub fn contains(&self, point: Point) -> bool {
        self.field.contains(point) || self.strip.contains(point)
    }

    pub fn slot_at(&self, point: Point) -> Option<&SwatchSlot> {
        self.slots.iter().find(|slot| slot.rect.contains(point))
    }
}

impl FrameLayout {
    pub fn place_editor(&mut self, text: &mut TextSystem, request: &EditorRequest) {
        self.editor = self
            .anchor_rect(request.anchor, request.target)
            .map(|anchor| layout_editor(text, &self.metrics, self.bounds, anchor, request));
    }

    fn anchor_rect(&self, anchor: EditAnchor, target: EditTarget) -> Option<Rect> {
        match (anchor, target) {
            (EditAnchor::Strip(pane), EditTarget::Tab(id)) => self
                .pane(pane)?
                .strip
                .tabs
                .iter()
                .find(|tab| tab.id == id)
                .map(|tab| tab.rect),
            (EditAnchor::Sidebar, target) => {
                let item = match target {
                    EditTarget::Instance(id) => SidebarItem::Instance(id),
                    EditTarget::Tab(id) => SidebarItem::Tab(id),
                };
                let sidebar = self.sidebar.as_ref()?;
                sidebar
                    .rows
                    .iter()
                    .find(|row| row.item == item)
                    .map(|row| row.rect)
            }
            _ => None,
        }
    }
}

fn layout_editor(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    bounds: Rect,
    anchor: Rect,
    request: &EditorRequest,
) -> EditorFrame {
    let row = swatch_row(metrics);
    let width = anchor.width.max(row.width).min(bounds.width);
    let x = anchor.x.min(bounds.right() - width).max(bounds.x).round();
    let inset = metrics.px(FIELD_INSET_Y);
    let field = Rect::new(x, anchor.y + inset, width, anchor.height - 2.0 * inset);
    let below = field.bottom() + row.height <= bounds.bottom();
    let strip_y = match below {
        true => field.bottom(),
        false => field.y - row.height,
    };
    let strip = Rect::new(x, strip_y, width, row.height);
    let text_area = field.inset_by(metrics.px(TEXT_PAD), 0.0);
    let style = metrics.label();
    let label = text.layout(request.field.text(), style);
    let caret_x = text.layout(request.field.before_caret(), style).width;
    let caret_width = metrics.line();
    let offset = (caret_x + caret_width - text_area.width).max(0.0);
    let text_x = text_area.x - offset;
    let caret_height = (label.ascent + label.descent).round();
    let caret = Rect::new(
        (text_x + caret_x).round(),
        (field.center().y - caret_height / 2.0).round(),
        caret_width,
        caret_height,
    );
    EditorFrame {
        target: request.target,
        field,
        text_area,
        label,
        text_x,
        caret,
        strip,
        slots: row.slots(metrics, strip),
        selected: request.color,
    }
}
