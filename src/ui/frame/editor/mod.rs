mod anchor;
mod emblems;
mod field;
mod form;
mod menu;
mod row;

use super::FrameLayout;
use crate::geometry::{Point, Rect};
use crate::input::TextField;
use crate::locale::Strings;
use crate::preferences::ConnectForm;
use crate::session::SessionId;
use crate::text::TextSystem;
use crate::theme::{Emblem, Swatch};
use crate::workspace::InstanceId;
pub use anchor::EditAnchor;
pub use emblems::EmblemSlot;
use emblems::{emblem_row, emblem_slots};
pub use field::FieldFrame;
pub(in crate::ui::frame) use field::place_field;
pub use form::{FormFrame, FormRow, SECRET_MASK};
use form::{FormSpot, place_form};
pub use menu::{MenuEntry, MenuFrame, MenuRequest, MenuRow};
pub use row::SwatchSlot;
use row::{RowSize, swatch_row, swatch_slots};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditTarget {
    Instance(InstanceId),
    Tab(SessionId),
}

#[derive(Clone, Copy, Debug)]
pub enum EditKind<'a> {
    Name(&'a TextField),
    Color(Option<Swatch>),
    Emblem(Option<Emblem>),
    Form(&'a ConnectForm, &'static Strings),
}

pub struct EditorRequest<'a> {
    pub target: EditTarget,
    pub anchor: EditAnchor,
    pub kind: EditKind<'a>,
}

#[derive(Clone, Debug)]
pub struct EditorFrame {
    pub target: EditTarget,
    pub field: Option<FieldFrame>,
    pub strip: Rect,
    pub slots: Vec<SwatchSlot>,
    pub selected: Option<Swatch>,
    pub emblems: Vec<EmblemSlot>,
    pub emblem: Option<Emblem>,
    pub form: Option<FormFrame>,
}

impl EditorFrame {
    fn empty(target: EditTarget) -> Self {
        Self {
            target,
            field: None,
            strip: Rect::default(),
            slots: Vec::new(),
            selected: None,
            emblems: Vec::new(),
            emblem: None,
            form: None,
        }
    }

    pub fn slot_at(&self, point: Point) -> Option<&SwatchSlot> {
        self.slots.iter().find(|slot| slot.rect.contains(point))
    }

    pub fn emblem_at(&self, point: Point) -> Option<&EmblemSlot> {
        self.emblems.iter().find(|slot| slot.rect.contains(point))
    }
}

impl FrameLayout {
    pub fn place_editor(&mut self, text: &mut TextSystem, request: &EditorRequest) {
        self.editor = self
            .anchor_rect(request.anchor, request.target)
            .map(|anchor| self.layout_editor(text, anchor, request));
    }

    fn layout_editor(
        &self,
        text: &mut TextSystem,
        anchor: Rect,
        request: &EditorRequest,
    ) -> EditorFrame {
        let metrics = &self.metrics;
        let editor = EditorFrame::empty(request.target);
        match request.kind {
            EditKind::Name(field) => EditorFrame {
                field: Some(place_field(text, metrics, anchor, self.bounds, field)),
                ..editor
            },
            EditKind::Color(selected) => {
                let strip = place_strip(anchor, swatch_row(metrics), self.bounds);
                let slots = swatch_slots(metrics, strip);
                EditorFrame {
                    strip,
                    slots,
                    selected,
                    ..editor
                }
            }
            EditKind::Form(form, strings) => {
                let spot = FormSpot {
                    anchor,
                    bounds: self.bounds,
                    form,
                    strings,
                };
                EditorFrame {
                    form: Some(place_form(text, metrics, &spot)),
                    ..editor
                }
            }
            EditKind::Emblem(emblem) => {
                let strip = place_strip(anchor, emblem_row(metrics), self.bounds);
                let emblems = emblem_slots(metrics, strip);
                EditorFrame {
                    strip,
                    emblems,
                    emblem,
                    ..editor
                }
            }
        }
    }
}

fn clamp_x(x: f32, width: f32, bounds: Rect) -> f32 {
    x.min(bounds.right() - width).max(bounds.x).round()
}

fn place_strip(anchor: Rect, row: RowSize, bounds: Rect) -> Rect {
    let width = anchor.width.max(row.width).min(bounds.width);
    let y = match anchor.bottom() + row.height <= bounds.bottom() {
        true => anchor.bottom(),
        false => anchor.y - row.height,
    };
    Rect::new(clamp_x(anchor.x, width, bounds), y, width, row.height)
}
