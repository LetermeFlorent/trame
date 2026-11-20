use winit::event::{ElementState, MouseButton as WinitButton};
use winit::window::Cursor;

use trame::geometry::Point;
use trame::ui::Hit;
use trame::ui::chrome::picture_close_rect;
use trame::workspace::PaneId;

use super::Running;
use super::actions::{
    PictureButtonAction, PictureButtonHit, PictureKeyAction, picture_button_action, picture_cursor,
    picture_key_action, picture_under_allowed,
};

impl Running {
    pub(in crate::app::running) fn picture_active(&self) -> bool {
        self.picture.is_some() && !self.frame.blocked && self.desk.closing.is_none()
    }

    pub(in crate::app::running) fn picture_button(
        &mut self,
        state: ElementState,
        button: WinitButton,
    ) -> bool {
        let blocked = self.frame.blocked || self.desk.closing.is_some();
        let hit = self.picture_hit();
        match picture_button_action(self.picture.is_some(), blocked, hit, state, button) {
            PictureButtonAction::Pass => false,
            PictureButtonAction::Consume => true,
            PictureButtonAction::OpenExternal => {
                self.open_picture_external();
                true
            }
            PictureButtonAction::Close => {
                self.close_picture();
                true
            }
        }
    }

    fn picture_hit(&self) -> PictureButtonHit {
        let under = self
            .pointer
            .position
            .and_then(|point| self.frame.hit(point, self.resizable()));
        match under {
            Some(Hit::Resize(_) | Hit::TitleBar | Hit::Minimize | Hit::Maximize | Hit::Close) => {
                return PictureButtonHit::WindowControl;
            }
            Some(Hit::SidebarToggle) => return PictureButtonHit::SidebarToggle,
            _ => {}
        }
        let on_open = self
            .picture
            .as_ref()
            .and_then(|p| p.frame.open)
            .zip(self.pointer.position)
            .is_some_and(|(rect, point)| rect.contains(point));
        match on_open {
            true => PictureButtonHit::OpenButton,
            false => PictureButtonHit::Other,
        }
    }

    pub(in crate::app::running) fn refresh_picture_hover(&mut self) {
        if self.view.hover.take().is_some() {
            self.window.request_redraw();
        }
        self.place_picture();
        let close = picture_close_rect(
            self.frame.bounds,
            self.frame.title_bar.rect.bottom(),
            &self.metrics,
        );
        if let Some(picture) = &mut self.picture
            && picture.sync_hover(close, self.pointer.position)
        {
            self.window.request_redraw();
        }
        let open = self.picture.as_ref().and_then(|p| p.frame.open);
        let cursor = Cursor::Icon(picture_cursor(close, open, self.pointer.position));
        if cursor != self.pointer.cursor {
            self.pointer.cursor = cursor.clone();
            self.window.set_cursor(cursor);
        }
    }

    pub(in crate::app::running) fn picture_key(&mut self, escape: bool) -> bool {
        let blocked = self.frame.blocked || self.desk.closing.is_some();
        match picture_key_action(self.picture.is_some(), blocked, escape) {
            PictureKeyAction::Pass => false,
            PictureKeyAction::Consume => true,
            PictureKeyAction::Close => {
                self.close_picture();
                true
            }
        }
    }

    pub(in crate::app::running::pointer) fn picture_under(
        &self,
        pane: PaneId,
        point: Point,
    ) -> bool {
        let selecting = self.selecting.drag.is_some() || self.selecting.held.is_some();
        let Some(pane) = self.frame.pane(pane) else {
            return false;
        };
        let Some(id) = pane.active else {
            return false;
        };
        if !picture_under_allowed(selecting, self.reports_mouse(id), self.primary_held()) {
            return false;
        }
        let cell = self.cell_in_pane(pane, point);
        self.desk.sessions.get(id).is_some_and(|session| {
            session
                .terminal()
                .screen()
                .image_at(cell.0, cell.1)
                .is_some()
        })
    }
}
