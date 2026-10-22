mod keys;
mod pointer;

use trame::input::TextField;
use trame::theme::{Emblem, Swatch};
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditTarget, EditorRequest, SidebarItem};

use super::Running;

pub(super) struct Editing {
    target: EditTarget,
    anchor: EditAnchor,
    field: TextField,
    original: String,
}

fn target_of(hit: Hit) -> Option<(EditTarget, EditAnchor)> {
    match hit {
        Hit::Tab(pane, id) => Some((EditTarget::Tab(id), EditAnchor::Strip(pane))),
        Hit::Row(SidebarItem::Tab(id)) => Some((EditTarget::Tab(id), EditAnchor::Sidebar)),
        Hit::Row(SidebarItem::Instance(id)) => {
            Some((EditTarget::Instance(id), EditAnchor::Sidebar))
        }
        Hit::RailInstance(id) => Some((EditTarget::Instance(id), EditAnchor::Rail)),
        _ => None,
    }
}

impl Running {
    pub(super) fn open_editor(&mut self, hit: Hit) -> bool {
        let Some((target, anchor)) = target_of(hit) else {
            return false;
        };
        let original = self.name_of(target);
        self.editing = Some(Editing {
            target,
            anchor,
            field: TextField::new(&original),
            original,
        });
        self.changed();
        true
    }

    pub(super) fn place_editor(&mut self) {
        let Some(editing) = &self.editing else {
            return;
        };
        let request = EditorRequest {
            target: editing.target,
            anchor: editing.anchor,
            field: &editing.field,
            color: self.color_of(editing.target),
            emblem: self.emblem_of(editing.target),
        };
        self.frame.place_editor(&mut self.text, &request);
        if self.frame.editor.is_none() {
            self.editing = None;
        }
    }

    fn commit_edit(&mut self) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        if editing.field.text() == editing.original {
            return;
        }
        let name = editing.field.text();
        match editing.target {
            EditTarget::Tab(id) => self.workspace.rename_tab(id, name),
            EditTarget::Instance(id) => self.workspace.rename_instance(id, name),
        }
    }

    fn cancel_edit(&mut self) {
        self.editing = None;
    }

    fn pick_color(&mut self, color: Option<Swatch>) {
        let Some(editing) = &self.editing else {
            return;
        };
        match editing.target {
            EditTarget::Tab(id) => self.workspace.set_tab_color(id, color),
            EditTarget::Instance(id) => self.workspace.set_instance_color(id, color),
        }
    }

    fn pick_emblem(&mut self, emblem: Emblem) {
        if let Some(EditTarget::Instance(id)) = self.editing.as_ref().map(|editing| editing.target)
        {
            self.workspace.set_instance_emblem(id, emblem);
        }
    }

    fn name_of(&self, target: EditTarget) -> String {
        match target {
            EditTarget::Tab(id) => self.tab_title(id),
            EditTarget::Instance(id) => self
                .workspace
                .instance(id)
                .map(|instance| instance.name.clone())
                .unwrap_or_default(),
        }
    }

    fn color_of(&self, target: EditTarget) -> Option<Swatch> {
        match target {
            EditTarget::Tab(id) => self.workspace.tab_color(id),
            EditTarget::Instance(id) => self.workspace.instance(id)?.color,
        }
    }

    fn emblem_of(&self, target: EditTarget) -> Option<Emblem> {
        match target {
            EditTarget::Tab(_) => None,
            EditTarget::Instance(id) => Some(self.workspace.instance(id)?.emblem),
        }
    }
}
