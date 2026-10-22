use super::{PaneFrame, SidebarFrame, SidebarItem};
use crate::theme::Swatch;
use crate::workspace::{InstanceId, Workspace};

pub fn apply(workspace: &Workspace, sidebar: Option<&mut SidebarFrame>, panes: &mut [PaneFrame]) {
    for tab in panes.iter_mut().flat_map(|pane| pane.strip.tabs.iter_mut()) {
        tab.color = workspace.tab_color(tab.id);
    }
    let Some(sidebar) = sidebar else {
        return;
    };
    for row in &mut sidebar.rows {
        row.color = match row.item {
            SidebarItem::Instance(id) => instance_color(workspace, id),
            SidebarItem::Tab(id) => workspace.tab_color(id),
        };
    }
}

fn instance_color(workspace: &Workspace, id: InstanceId) -> Option<Swatch> {
    workspace
        .instances
        .iter()
        .find(|instance| instance.id == id)
        .and_then(|instance| instance.color)
}
