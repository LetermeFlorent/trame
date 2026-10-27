mod moves;
mod splits;
mod swaps;

use trame::workspace::{Edge, Workspace};

use crate::{tab, workspace_with};

fn two_panes() -> Workspace {
    let mut workspace = workspace_with(&[1, 2, 3]);
    workspace.split_active(tab(4), Edge::Right);
    workspace
}

fn two_instances() -> Workspace {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.add_instance(None, tab(3));
    workspace
}
