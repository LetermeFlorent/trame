use std::collections::HashMap;

use winit::dpi::PhysicalPosition;

use trame::geometry::{Point, Rect};
use trame::session::SessionId;
use trame::workspace::{InstanceId, MovingInstance, TabMark};

use crate::app::running::Running;
use crate::app::sessions::Leaving;

#[derive(Clone, Copy)]
pub enum Lifted {
    Tab(SessionId),
    Instance(InstanceId),
}

pub struct TornTab {
    leaving: Leaving,
    mark: TabMark,
}

pub struct MovedInstance {
    moving: MovingInstance,
    sessions: Vec<(SessionId, Leaving)>,
}

impl Running {
    pub fn can_tear(&self, tab: SessionId) -> bool {
        self.workspace.locate(tab).is_some() && !self.workspace.is_only_tab(tab)
    }

    pub fn tear_tab(&mut self, tab: SessionId) -> Option<TornTab> {
        let mark = self.workspace.take_tab(tab)?;
        let leaving = self.sessions.hand_off(tab);
        self.after_close();
        Some(TornTab { leaving, mark })
    }

    pub fn give_instance(&mut self, id: InstanceId) -> Option<MovedInstance> {
        let moving = self.workspace.take_instance(id)?;
        let tabs = moving.instance.tabs();
        let sessions = tabs
            .into_iter()
            .map(|tab| (tab, self.sessions.hand_off(tab)))
            .collect();
        self.after_close();
        Some(MovedInstance { moving, sessions })
    }

    pub(in crate::app::running) fn take_torn_tab(&mut self, torn: TornTab) {
        let id = self.sessions.take_in(torn.leaving);
        self.workspace.add_instance(None, id);
        self.workspace.mark_tab(id, torn.mark);
        self.relayout();
        self.start_shown();
        self.changed();
    }

    pub fn take_dropped_tab(&mut self, torn: TornTab, at: PhysicalPosition<i32>) {
        let id = self.sessions.take_in(torn.leaving);
        let point = self.client_point(at);
        if let Some(index) = point.and_then(|point| self.instance_target(point)) {
            self.workspace.active = index;
        }
        self.workspace.add_tab(id);
        self.workspace.mark_tab(id, torn.mark);
        let instance = self.workspace.active_instance().map(|instance| instance.id);
        let spot = point
            .zip(instance)
            .and_then(|(point, instance)| self.frame.drop_at(point, instance));
        if let Some(spot) = spot {
            self.workspace.move_tab(id, spot.target);
        }
        self.workspace.activate_tab(id);
        self.welcome();
    }

    pub fn take_moved_instance(&mut self, moved: MovedInstance, at: PhysicalPosition<i32>) {
        let renamed: HashMap<SessionId, SessionId> = moved
            .sessions
            .into_iter()
            .map(|(old, leaving)| (old, self.sessions.take_in(leaving)))
            .collect();
        let index = self
            .client_point(at)
            .and_then(|point| self.instance_target(point));
        let index = index.unwrap_or(self.workspace.instances.len());
        self.workspace.adopt_instance(moved.moving, &renamed, index);
        self.welcome();
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

    fn welcome(&mut self) {
        self.motion.animate_next();
        self.relayout();
        self.start_shown();
        self.changed();
        self.bring_forward();
    }
}
