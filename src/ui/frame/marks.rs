use super::{FrameLayout, PaneFrame, SidebarFrame, SidebarItem};
use crate::session::SessionId;
use crate::theme::{Emblem, Swatch};
use crate::workspace::{InstanceId, Workspace};

pub fn apply(workspace: &Workspace, sidebar: Option<&mut SidebarFrame>, panes: &mut [PaneFrame]) {
    for pane in panes.iter_mut() {
        pane.tint = pane.active.and_then(|id| workspace.tint_of(id));
    }
    for tab in panes.iter_mut().flat_map(|pane| pane.strip.tabs.iter_mut()) {
        tab.color = workspace.tab_color(tab.id);
        tab.fill = workspace.tab_fill(tab.id);
    }
    let Some(sidebar) = sidebar else {
        return;
    };
    for row in &mut sidebar.rows {
        (row.color, row.emblem) = match row.item {
            SidebarItem::Instance(id) => instance_marks(workspace, id),
            SidebarItem::Tab(id) => (workspace.tab_color(id), None),
        };
        row.fill = match row.item {
            SidebarItem::Instance(id) => workspace.instance(id).is_some_and(|item| item.fill),
            SidebarItem::Tab(_) => false,
        };
    }
}

fn instance_marks(workspace: &Workspace, id: InstanceId) -> (Option<Swatch>, Option<Emblem>) {
    workspace.instance(id).map_or((None, None), |instance| {
        (instance.color, Some(workspace.shown_emblem(instance)))
    })
}

impl FrameLayout {
    pub fn mark_working(&mut self, workspace: &Workspace, working: impl Fn(SessionId) -> bool) {
        for tab in self
            .panes
            .iter_mut()
            .flat_map(|pane| pane.strip.tabs.iter_mut())
        {
            tab.working = working(tab.id);
        }
        let Some(sidebar) = self.sidebar.as_mut() else {
            return;
        };
        for row in &mut sidebar.rows {
            row.working = match row.item {
                SidebarItem::Tab(id) => working(id),
                SidebarItem::Instance(id) => workspace
                    .instance(id)
                    .is_some_and(|instance| instance.tabs().into_iter().any(&working)),
            };
        }
    }

    pub fn shows_work(&self) -> bool {
        let tabs = self.panes.iter().flat_map(|pane| &pane.strip.tabs);
        let rows = self.sidebar.iter().flat_map(|sidebar| &sidebar.rows);
        tabs.into_iter().any(|tab| tab.working) || rows.into_iter().any(|row| row.working)
    }
}
