use std::path::Path;

use trame::files::{home_dir, short_path};
use trame::input::Command;
use trame::ui::frame::{
    ColorsView, EditAnchor, EditKind, EditTarget, EditorRequest, EmblemView, FormAction, MenuEntry,
    MenuRequest, MessageRequest,
};
use trame::workspace::PaneId;

use super::{EditMode, Menu, edit_entries};
use crate::gnome::state::WindowState;

const FOLDER_ACTIONS: [FormAction; 2] = [FormAction::Change, FormAction::Apply];

impl WindowState {
    pub fn place_editor(&mut self) {
        let Some(editing) = &self.editing else {
            return;
        };
        let kind = match &editing.mode {
            EditMode::Folder { chosen, .. } => {
                let chosen = chosen.clone();
                return self.place_folder_prompt(chosen.as_deref());
            }
            EditMode::Text { field, .. } => EditKind::Name(field),
            EditMode::Form(panel) => EditKind::Connect(panel.view()),
            EditMode::Color { first } => EditKind::Color(ColorsView {
                selected: self.desk.color_of(editing.target),
                custom: &self.desk.preferences.custom_colors,
                first: *first,
                filled: self.desk.fill_of(editing.target),
            }),
            EditMode::Picker(picker) => EditKind::Picker(picker.view()),
            EditMode::Emblem { letters } => EditKind::Emblem(EmblemView {
                selected: self.desk.emblem_of(editing.target),
                letters,
            }),
        };
        let req = EditorRequest {
            target: editing.target,
            anchor: editing.anchor,
            kind,
            strings: self.desk.strings,
        };
        self.frame.place_editor(&mut self.text, &req);
        if self.frame.editor.is_none() {
            self.editing = None;
        }
    }

    fn place_folder_prompt(&mut self, chosen: Option<&Path>) {
        let shown = chosen.map(|p| short_path(p, home_dir().as_deref()));
        let strings = self.desk.strings;
        let lines = [shown.as_deref().unwrap_or(strings.pick_folder)];
        let req = MessageRequest {
            title: strings.change_folder,
            lines: &lines,
            actions: &FOLDER_ACTIONS,
            strings,
        };
        self.frame.place_message(&mut self.text, &req);
    }

    pub fn place_menu(&mut self) {
        let Some(menu) = &self.menu else {
            return;
        };
        let entries = match menu.adding {
            true => MenuEntry::adding(self.desk.strings).to_vec(),
            false => edit_entries(menu.target, self.desk.strings),
        };
        let req = MenuRequest {
            at: menu.at,
            entries: &entries,
        };
        self.frame.place_menu(&mut self.text, &req);
    }

    pub fn open_adding_menu(&mut self, exiting: bool, pane: PaneId) {
        self.desk.focus_pane(pane);
        let found = self
            .frame
            .panes
            .iter()
            .find(|f| f.id == pane)
            .and_then(|f| Some((f.active?, f.adding_menu_at())));
        let Some((tab, at)) = found else {
            return self.run_command(exiting, Command::NewTab);
        };
        self.editing = None;
        self.menu = Some(Menu {
            target: EditTarget::Tab(tab),
            anchor: EditAnchor::Strip(pane),
            at,
            adding: true,
        });
        self.changed();
    }
}
