use trame::files::{PickRequest, import_icon, pick_image};
use trame::input::TextField;
use trame::log_error;
use trame::theme::{Emblem, MAX_LETTERS};
use trame::ui::frame::EditTarget;
use trame::workspace::InstanceId;

use super::{EditMode, Editing, Running};
use crate::app::{UserEvent, window};

impl Editing {
    pub(super) fn letters_mut(&mut self) -> Option<&mut TextField> {
        match &mut self.mode {
            EditMode::Emblem { letters } => Some(letters),
            _ => None,
        }
    }
}

pub(super) fn edit_letters(field: &mut TextField, change: impl FnOnce(&mut TextField)) {
    let before = field.clone();
    change(field);
    if field.text().chars().count() > MAX_LETTERS {
        *field = before;
    }
}

impl Running {
    pub(super) fn emblem_mode(&self, target: EditTarget) -> EditMode {
        let shown = match self.emblem_of(target) {
            Some(Emblem::Letters(letters)) => letters.to_string(),
            _ => String::new(),
        };
        let mut letters = TextField::new(&shown);
        letters.select_all();
        EditMode::Emblem { letters }
    }

    pub(super) fn emblem_of(&self, target: EditTarget) -> Option<Emblem> {
        match target {
            EditTarget::Tab(_) => None,
            EditTarget::Instance(id) => Some(self.workspace.instance(id)?.emblem),
        }
    }

    pub(super) fn set_emblem(&mut self, target: EditTarget, emblem: Emblem) {
        if let EditTarget::Instance(id) = target {
            self.workspace.set_instance_emblem(id, emblem);
        }
    }

    pub(super) fn ask_image(&self) {
        let Some(EditTarget::Instance(id)) = self.editing.as_ref().map(|editing| editing.target)
        else {
            return;
        };
        let Some(dir) = self.icons.dir().map(ToOwned::to_owned) else {
            log_error!("there is no folder to keep icons in");
            return;
        };
        let request = PickRequest {
            owner: window::owner(&self.window),
            title: self.strings.pick_image.to_owned(),
            start: None,
        };
        let proxy = self.proxy.clone();
        pick_image(request, move |path| match import_icon(&path, &dir) {
            Ok(image) => {
                let _ = proxy.send_event(UserEvent::Icon(id, image));
            }
            Err(error) => log_error!("the image could not become an icon: {error}"),
        });
    }

    pub fn icon_picked(&mut self, id: InstanceId, image: u64) {
        self.icons.reload(image);
        let target = EditTarget::Instance(id);
        if self
            .editing
            .as_ref()
            .is_some_and(|editing| editing.target == target)
        {
            self.editing = None;
        }
        self.set_emblem(target, Emblem::Image(image));
        self.changed();
    }
}
