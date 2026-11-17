use crate::theme::{Emblem, Swatch};
use crate::workspace::{EditTarget, InstanceId};

use super::super::{Desk, Effect, Effects};

impl Desk {
    pub fn color_of(&self, target: EditTarget) -> Option<Swatch> {
        match target {
            EditTarget::Tab(id) => self.workspace.tab_color(id),
            EditTarget::Instance(id) => self.workspace.instance(id)?.color,
        }
    }

    pub fn fill_of(&self, target: EditTarget) -> bool {
        match target {
            EditTarget::Tab(id) => self.workspace.tab_fill(id),
            EditTarget::Instance(id) => self.workspace.instance(id).is_some_and(|item| item.fill),
        }
    }

    pub fn emblem_of(&self, target: EditTarget) -> Option<Emblem> {
        match target {
            EditTarget::Tab(_) => None,
            EditTarget::Instance(id) => Some(self.workspace.instance(id)?.emblem),
        }
    }

    pub fn forget_custom_color(&mut self, index: usize) -> Effects {
        self.store_preferences(self.preferences.without_custom_color(index))
    }

    pub fn icon_picked(&mut self, id: InstanceId, image: u64) -> Effects {
        self.icons.reload(image);
        self.set_emblem(EditTarget::Instance(id), Emblem::Image(image));
        Effect::Changed.into()
    }
}
