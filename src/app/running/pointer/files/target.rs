use trame::ui::Hit;
use trame::workspace::PaneId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DropTarget {
    Ignored,
    Focused,
    Pane(PaneId),
}

pub(super) fn drop_target(hit: Option<Hit>, modal: bool) -> DropTarget {
    match (modal, hit) {
        (true, _) => DropTarget::Ignored,
        (false, Some(Hit::Terminal(pane))) => DropTarget::Pane(pane),
        (false, _) => DropTarget::Focused,
    }
}
