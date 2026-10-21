use crate::workspace::Edge;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    NewTab,
    CloseTab,
    NewInstance,
    SplitRight,
    SplitDown,
    ToggleSidebar,
    Copy,
    Paste,
    NextTab,
    PreviousTab,
    SelectTab(usize),
    LastTab,
    NextInstance,
    PreviousInstance,
    FocusPane(Edge),
    FontBigger,
    FontSmaller,
    FontReset,
    ScrollPageUp,
    ScrollPageDown,
    ScrollTop,
    ScrollBottom,
}
