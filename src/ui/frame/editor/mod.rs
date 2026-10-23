mod anchor;
mod field;
mod form;
mod menu;
mod popups;

use super::{FrameLayout, HostList, HostsView, PopupFrame, SettingLine};
use crate::geometry::{Point, Rect};
use crate::input::TextField;
use crate::locale::Strings;
use crate::session::SessionId;
use crate::text::TextSystem;
use crate::theme::{Emblem, Swatch};
use crate::workspace::InstanceId;
use anchor::AnchorSpot;
pub use anchor::EditAnchor;
pub use field::FieldFrame;
pub(in crate::ui::frame) use field::place_field;
use field::place_inline;
pub use form::{FormAction, FormButton, FormFrame, FormRow, SECRET_MASK};
pub(in crate::ui::frame) use form::{FormSpec, button_form, form_height, form_title, place_form};
pub use menu::{MenuEntry, MenuFrame, MenuRequest, MenuRow};
use popups::ConnectDialog;
pub use popups::{EmblemSlot, SwatchSlot};

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
    Connect(HostsView<'a>),
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
    pub list: Option<HostList>,
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
            list: None,
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
            .anchor_spot(request.anchor, request.target)
            .map(|spot| self.layout_editor(text, spot, request));
    }

    fn layout_editor(
        &self,
        text: &mut TextSystem,
        spot: AnchorSpot,
        request: &EditorRequest,
    ) -> EditorFrame {
        let editor = EditorFrame::empty(request.target, request.anchor);
        let strings = request.strings;
        match request.kind {
            EditKind::Name(field) => EditorFrame {
                field: Some(self.name_field(text, spot, field)),
                ..editor
            },
            EditKind::Color(selected) => self.color_popup(text, strings, selected, editor),
            EditKind::Emblem(emblem) => self.emblem_popup(text, strings, emblem, editor),
            EditKind::Connect(view) => {
                let dialog = ConnectDialog { view, strings };
                self.connect_popup(text, &dialog, editor)
            }
        }
    }
}

impl FrameLayout {
    fn name_field(&self, text: &mut TextSystem, spot: AnchorSpot, field: &TextField) -> FieldFrame {
        let metrics = &self.metrics;
        match spot.label {
            Some(label) => place_inline(text, metrics, spot.rect, label, field),
            None => place_field(text, metrics, spot.rect, self.bounds, field),
        }
    }
}

fn clamp_x(x: f32, width: f32, bounds: Rect) -> f32 {
    x.min(bounds.right() - width).max(bounds.x).round()
}
