use crate::input::Command;
use crate::workspace::Edge;

const ACTION_COUNT: usize = 23;

const TABLE: [(Command, &str); ACTION_COUNT] = [
    (Command::NewTab, "new_tab"),
    (Command::CloseTab, "close_tab"),
    (Command::NextTab, "next_tab"),
    (Command::PreviousTab, "previous_tab"),
    (Command::NewInstance, "new_instance"),
    (Command::NextInstance, "next_instance"),
    (Command::PreviousInstance, "previous_instance"),
    (Command::SplitRight, "split_right"),
    (Command::SplitDown, "split_down"),
    (Command::FocusPane(Edge::Left), "focus_left"),
    (Command::FocusPane(Edge::Right), "focus_right"),
    (Command::FocusPane(Edge::Top), "focus_up"),
    (Command::FocusPane(Edge::Bottom), "focus_down"),
    (Command::ToggleSidebar, "toggle_sidebar"),
    (Command::Copy, "copy"),
    (Command::Paste, "paste"),
    (Command::FontBigger, "font_bigger"),
    (Command::FontSmaller, "font_smaller"),
    (Command::FontReset, "font_reset"),
    (Command::ScrollPageUp, "scroll_page_up"),
    (Command::ScrollPageDown, "scroll_page_down"),
    (Command::ScrollTop, "scroll_top"),
    (Command::ScrollBottom, "scroll_bottom"),
];

pub const ACTIONS: [Command; ACTION_COUNT] = {
    let mut commands = [Command::NewTab; ACTION_COUNT];
    let mut index = 0;
    while index < ACTION_COUNT {
        commands[index] = TABLE[index].0;
        index += 1;
    }
    commands
};

pub fn action_id(action: usize) -> &'static str {
    TABLE[action].1
}

pub fn action_of(id: &str) -> Option<usize> {
    TABLE.iter().position(|(_, known)| *known == id)
}
