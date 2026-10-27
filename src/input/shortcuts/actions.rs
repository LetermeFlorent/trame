use crate::input::Command;
use crate::workspace::Edge;

pub const ACTIONS: [Command; 32] = [
    Command::NewTab,
    Command::CloseTab,
    Command::NextTab,
    Command::PreviousTab,
    Command::SelectTab(0),
    Command::SelectTab(1),
    Command::SelectTab(2),
    Command::SelectTab(3),
    Command::SelectTab(4),
    Command::SelectTab(5),
    Command::SelectTab(6),
    Command::SelectTab(7),
    Command::LastTab,
    Command::NewInstance,
    Command::NextInstance,
    Command::PreviousInstance,
    Command::SplitRight,
    Command::SplitDown,
    Command::FocusPane(Edge::Left),
    Command::FocusPane(Edge::Right),
    Command::FocusPane(Edge::Top),
    Command::FocusPane(Edge::Bottom),
    Command::ToggleSidebar,
    Command::Copy,
    Command::Paste,
    Command::FontBigger,
    Command::FontSmaller,
    Command::FontReset,
    Command::ScrollPageUp,
    Command::ScrollPageDown,
    Command::ScrollTop,
    Command::ScrollBottom,
];

const IDS: [&str; 32] = [
    "new_tab",
    "close_tab",
    "next_tab",
    "previous_tab",
    "tab_1",
    "tab_2",
    "tab_3",
    "tab_4",
    "tab_5",
    "tab_6",
    "tab_7",
    "tab_8",
    "last_tab",
    "new_instance",
    "next_instance",
    "previous_instance",
    "split_right",
    "split_down",
    "focus_left",
    "focus_right",
    "focus_up",
    "focus_down",
    "toggle_sidebar",
    "copy",
    "paste",
    "font_bigger",
    "font_smaller",
    "font_reset",
    "scroll_page_up",
    "scroll_page_down",
    "scroll_top",
    "scroll_bottom",
];

pub fn action_id(action: usize) -> &'static str {
    IDS[action]
}

pub fn action_of(id: &str) -> Option<usize> {
    IDS.iter().position(|known| *known == id)
}
