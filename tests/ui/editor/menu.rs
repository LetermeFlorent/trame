use trame::geometry::Point;
use trame::text::TextSystem;
use trame::ui::frame::{MenuEntry, MenuRequest};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use crate::{HEIGHT, WIDTH, frame, text_system, workspace};

const ENTRIES: [(MenuEntry, &str); 4] = [
    (MenuEntry::Rename, "Rename"),
    (MenuEntry::Color, "Change colour"),
    (MenuEntry::Icon, "Change icon"),
    (MenuEntry::Folder, "Change folder"),
];

fn menu_at(text: &mut TextSystem, workspace: &Workspace, at: Point) -> FrameLayout {
    let mut frame = frame(text, workspace);
    frame.place_menu(
        text,
        &MenuRequest {
            at,
            entries: &ENTRIES,
        },
    );
    frame
}

#[test]
fn no_menu_is_placed_until_one_is_requested() {
    let mut text = text_system();
    assert!(frame(&mut text, &workspace()).menu.is_none());
}

#[test]
fn the_menu_opens_at_the_pointer_with_one_row_per_entry_in_order() {
    let mut text = text_system();
    let at = Point::new(300.0, 200.0);
    let frame = menu_at(&mut text, &workspace(), at);
    let menu = frame.menu.as_ref().expect("a menu");
    assert_eq!((menu.rect.x, menu.rect.y), (at.x, at.y));
    let order: Vec<MenuEntry> = menu.rows.iter().map(|row| row.entry).collect();
    assert_eq!(order, ENTRIES.map(|(entry, _)| entry).to_vec());
    for pair in menu.rows.windows(2) {
        assert_eq!(pair[1].rect.y, pair[0].rect.bottom());
    }
    for row in &menu.rows {
        assert!(menu.rect.contains(row.rect.center()));
        assert!(row.label_x + row.label.width <= row.rect.right());
        assert_eq!(
            frame.hit(row.rect.center(), false),
            Some(Hit::MenuEntry(row.entry))
        );
    }
    let corner = Point::new(menu.rect.x + 1.0, menu.rect.y + 1.0);
    assert_eq!(frame.hit(corner, false), Some(Hit::Menu));
}

#[test]
fn the_menu_covers_what_lies_under_it() {
    let mut text = text_system();
    let workspace = workspace();
    let tab = frame(&mut text, &workspace).panes[0].strip.tabs[0].rect;
    let frame = menu_at(&mut text, &workspace, tab.center());
    let first = frame.menu.as_ref().expect("a menu").rows[0].rect.center();
    let plain = crate::frame(&mut text, &workspace);
    assert_ne!(
        plain.hit(first, false),
        Some(Hit::MenuEntry(MenuEntry::Rename))
    );
    assert_eq!(
        frame.hit(first, false),
        Some(Hit::MenuEntry(MenuEntry::Rename))
    );
}

#[test]
fn near_the_corner_the_menu_stays_inside_the_window() {
    let mut text = text_system();
    let frame = menu_at(
        &mut text,
        &workspace(),
        Point::new(WIDTH - 2.0, HEIGHT - 2.0),
    );
    let menu = frame.menu.as_ref().expect("a menu");
    assert!(menu.rect.right() <= WIDTH);
    assert!(menu.rect.bottom() <= HEIGHT);
    assert!(menu.rect.x >= 0.0 && menu.rect.y >= 0.0);
}
