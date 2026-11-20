use trame::desk::{Closing, Confirm};
use trame::input::Command;
use trame::ui::Hit;
use trame::ui::frame::{BarPart, EditAnchor, FormAction, MenuEntry, MessageRequest, SidebarItem};
use trame::workspace::PaneId;

use crate::gnome::state::editing::{Menu, target_of};
use crate::gnome::state::settings::ListKind;
use crate::gnome::state::{WindowAction, WindowState};

const CLOSE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Close];
const UPDATE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Restart];
const REMOVE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Remove];
const PASTE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Paste];

impl WindowState {
    pub(super) fn click(&mut self, exiting: bool, hit: Hit) {
        match hit {
            Hit::SidebarToggle => self.run_command(exiting, Command::ToggleSidebar),
            Hit::Minimize => self.act(WindowAction::Minimize),
            Hit::Maximize => self.act(WindowAction::ToggleMaximize),
            Hit::Close => self.ask_close(Closing::Window),
            Hit::NewInstance => self.run_command(exiting, Command::NewInstance),
            Hit::Settings => self.toggle_settings(),
            Hit::Update => self.ask_close(Closing::Update),
            Hit::Setting(c) => self.apply_setting(c),
            Hit::List(c) => self.list_control(c),
            Hit::SidebarBar(BarPart::Track) => self.page_list(ListKind::Sidebar),
            Hit::FormField(f) => self.focus_form(f),
            Hit::FormButton(a) => self.form_action(a),
            Hit::Disconnect(tab) => {
                let effects = self.desk.disconnect(tab);
                self.apply(effects);
            }
            Hit::RowClose(SidebarItem::Instance(id)) => self.ask_close(Closing::Instance(id)),
            Hit::RowClose(SidebarItem::Tab(id)) | Hit::TabClose(_, id) => {
                let effects = self.desk.close_tab(exiting, id);
                self.apply(effects);
            }
            Hit::RowFold(id) => self.desk.workspace.toggle_folded(id),
            Hit::NewTab(pane) => self.open_adding_menu(exiting, pane),
            Hit::Split(pane) => self.split_pane(exiting, pane),
            _ => {}
        }
    }

    fn split_pane(&mut self, exiting: bool, pane: PaneId) {
        self.desk.focus_pane(pane);
        let cmd = if self.modifiers.shift_key() {
            Command::SplitDown
        } else {
            Command::SplitRight
        };
        self.run_command(exiting, cmd);
    }

    pub fn place_message(&mut self) {
        let Some(q) = self.desk.question() else {
            self.desk.closing = None;
            return;
        };
        let lines: Vec<&str> = q.body.lines().collect();
        let actions: &[FormAction] = match q.confirm {
            Confirm::Close => &CLOSE_ACTIONS,
            Confirm::Restart => &UPDATE_ACTIONS,
            Confirm::Remove => &REMOVE_ACTIONS,
            Confirm::Paste => &PASTE_ACTIONS,
        };
        let req = MessageRequest {
            title: &q.title,
            lines: &lines,
            actions,
            strings: self.desk.strings,
        };
        self.frame.place_message(&mut self.text, &req);
    }

    pub(super) fn closing_button(&mut self, pressed: bool) -> bool {
        if self.desk.closing.is_none() {
            return false;
        }
        let hit = self.pointer.and_then(|p| self.frame.hit(p, false));
        if !matches!(
            hit,
            Some(Hit::FormButton(_) | Hit::EditClose | Hit::Dialog | Hit::Backdrop)
        ) {
            return false;
        }
        if pressed && hit != Some(Hit::Dialog) {
            let ok = matches!(
                hit,
                Some(Hit::FormButton(
                    FormAction::Close
                        | FormAction::Restart
                        | FormAction::Remove
                        | FormAction::Paste
                ))
            );
            let effects = self.desk.settle_closing(ok);
            self.apply(effects);
        }
        true
    }

    pub(super) fn menu_button(&mut self, exiting: bool, pressed: bool, button: u32) -> bool {
        let Some(point) = self.pointer.filter(|_| pressed) else {
            return false;
        };
        let hit = self.frame.hit(point, false);
        let was_open = self.menu.is_some();
        if let Some(menu) = self.menu.take() {
            self.apply_menu_hit(exiting, menu, hit);
        }
        if button == 3
            && let Some((target, anchor)) = hit.and_then(target_of)
        {
            self.editing = None;
            self.menu = Some(Menu {
                target,
                anchor,
                at: point,
                adding: false,
            });
            self.changed();
            return true;
        }
        if was_open {
            self.changed();
        }
        was_open
    }

    fn apply_menu_hit(&mut self, exiting: bool, menu: Menu, hit: Option<Hit>) {
        match hit {
            Some(Hit::Menu) => self.menu = Some(menu),
            Some(Hit::MenuEntry(MenuEntry::NewTab)) => {
                if let EditAnchor::Strip(p) = menu.anchor {
                    self.desk.focus_pane(p);
                }
                self.run_command(exiting, Command::NewTab);
            }
            Some(Hit::MenuEntry(e)) => self.start_edit(menu.target, menu.anchor, e),
            _ => {}
        }
    }
}
