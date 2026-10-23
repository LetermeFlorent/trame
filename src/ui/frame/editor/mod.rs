mod anchor;
mod dialogs;
mod emblems;
mod field;
mod form;
mod menu;
mod row;

use super::{FrameLayout, PopupFrame, SettingLine};
use crate::geometry::{Point, Rect};
use crate::input::TextField;
use crate::locale::Strings;
use crate::preferences::{ConnectForm, SshHost};
use crate::session::SessionId;
use crate::text::TextSystem;
use crate::theme::{Emblem, Swatch};
use crate::workspace::InstanceId;
pub use anchor::EditAnchor;
use dialogs::ConnectDialog;
pub use emblems::EmblemSlot;
pub use field::FieldFrame;
pub(in crate::ui::frame) use field::place_field;
pub use form::{FormFrame, FormRow, SECRET_MASK};
pub use menu::{MenuEntry, MenuFrame, MenuRequest, MenuRow};
pub use row::SwatchSlot;

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
    Form(&'a ConnectForm, &'a [SshHost]),
}

pub struct EditorRequest<'a> {
    pub target: EditTarget,
    pub anchor: EditAnchor,
    pub kind: EditKind<'a>,
    pub strings: &'static Strings,
}

#[derive(Clone, Debug)]
pub struct EditorFrame {
    pub target: EditTarget,
    pub anchor: EditAnchor,
    pub field: Option<FieldFrame>,
    pub popup: Option<PopupFrame>,
    pub strip: Rect,
    pub slots: Vec<SwatchSlot>,
    pub selected: Option<Swatch>,
    pub emblems: Vec<EmblemSlot>,
    pub emblem: Option<Emblem>,
    pub sections: Vec<SettingLine>,
    pub hosts: Vec<SettingLine>,
    pub form: Option<FormFrame>,
}

impl EditorFrame {
    fn empty(target: EditTarget, anchor: EditAnchor) -> Self {
        Self {
            target,
            anchor,
            field: None,
            popup: None,
            strip: Rect::default(),
            slots: Vec::new(),
            selected: None,
            emblems: Vec::new(),
            emblem: None,
            sections: Vec::new(),
            hosts: Vec::new(),
            form: None,
        }
    }

    pub fn slot_at(&self, point: Point) -> Option<&SwatchSlot> {
        self.slots.iter().find(|slot| slot.rect.contains(point))
    }

    pub fn emblem_at(&self, point: Point) -> Option<&EmblemSlot> {
        self.emblems.iter().find(|slot| slot.rect.contains(point))
    }

    pub fn host_at(&self, point: Point) -> Option<usize> {
        self.hosts.iter().position(|line| line.rect.contains(point))
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
        let editor = EditorFrame::empty(request.target, request.anchor);
        let strings = request.strings;
        match request.kind {
            EditKind::Name(field) => EditorFrame {
                field: Some(place_field(text, &self.metrics, anchor, self.bounds, field)),
                ..editor
            },
            EditKind::Color(selected) => self.color_popup(text, strings, selected, editor),
            EditKind::Emblem(emblem) => self.emblem_popup(text, strings, emblem, editor),
            EditKind::Form(form, hosts) => {
                let dialog = ConnectDialog {
                    form,
                    hosts,
                    strings,
                };
                self.connect_popup(text, &dialog, editor)
            }
        }
    }
}

fn clamp_x(x: f32, width: f32, bounds: Rect) -> f32 {
    x.min(bounds.right() - width).max(bounds.x).round()
}
