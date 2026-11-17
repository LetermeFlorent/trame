use winit::dpi::PhysicalPosition;

use trame::desk::{MovedInstance, TornTab};
use trame::geometry::{Point, Rect};
use trame::session::SessionId;
use trame::workspace::InstanceId;

use crate::app::running::Running;

#[derive(Clone, Copy)]
pub enum Lifted {
    Tab(SessionId),
    Instance(InstanceId),
}

impl Running {
    pub fn can_tear(&self, tab: SessionId) -> bool {
        self.desk.can_tear(tab)
    }

    pub fn tear_tab(&mut self, tab: SessionId) -> Option<TornTab> {
        let outcome = self.desk.tear_tab(tab);
        self.apply(outcome.effects);
        outcome.value
    }

    pub fn give_instance(&mut self, id: InstanceId) -> Option<MovedInstance> {
        let outcome = self.desk.give_instance(id);
        self.apply(outcome.effects);
        outcome.value
    }

    pub(in crate::app::running) fn take_torn_tab(&mut self, torn: TornTab) {
        let effects = self.desk.adopt_torn(torn);
        self.apply(effects);
    }

    pub fn take_dropped_tab(&mut self, torn: TornTab, at: PhysicalPosition<i32>) {
        let point = self.client_point(at);
        let index = point.and_then(|point| self.instance_target(point));
        let id = self.desk.receive_tab(torn, index);
        let instance = self
            .desk
            .workspace
            .active_instance()
            .map(|instance| instance.id);
        let spot = point
            .zip(instance)
            .and_then(|(point, instance)| self.frame.drop_at(point, instance));
        if let Some(spot) = spot {
            self.desk.workspace.move_tab(id, spot.target);
        }
        self.desk.workspace.activate_tab(id);
        let effects = self.desk.welcome();
        self.apply(effects);
    }

    pub fn take_moved_instance(&mut self, moved: MovedInstance, at: PhysicalPosition<i32>) {
        let index = self
            .client_point(at)
            .and_then(|point| self.instance_target(point));
        let effects = self.desk.receive_instance(moved, index);
        self.apply(effects);
    }

    pub fn holds(&self, at: PhysicalPosition<i32>) -> bool {
        let size = self.window.inner_size();
        let window = Rect::new(0.0, 0.0, size.width as f32, size.height as f32);
        !self.hidden()
            && self.window.is_minimized() != Some(true)
            && self
                .client_point(at)
                .is_some_and(|point| window.contains(point))
    }

    fn client_point(&self, at: PhysicalPosition<i32>) -> Option<Point> {
        let origin = self.window.inner_position().ok()?;
        Some(Point::new(
            (at.x - origin.x) as f32,
            (at.y - origin.y) as f32,
        ))
    }

    pub(in crate::app::running) fn screen_point(
        &self,
        point: Point,
    ) -> Option<PhysicalPosition<i32>> {
        let origin = self.window.inner_position().ok()?;
        let x = origin.x + point.x.round() as i32;
        let y = origin.y + point.y.round() as i32;
        Some(PhysicalPosition::new(x, y))
    }
}
