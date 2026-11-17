use trame::desk::{FALLBACK_GRID, Grids};
use trame::session::SessionId;

#[test]
fn unknown_tabs_get_the_fallback_grid() {
    let grids = Grids::default();
    assert_eq!(grids.size_of(SessionId(7)), FALLBACK_GRID);
    assert!(grids.shown().is_empty());
}

#[test]
fn panes_give_their_size_to_every_tab() {
    let mut grids = Grids::default();
    grids.add_pane(&[SessionId(1), SessionId(2)], Some(SessionId(2)), (30, 100));
    grids.add_pane(&[SessionId(3)], Some(SessionId(3)), (10, 40));
    grids.add_pane(&[], None, (5, 5));
    assert_eq!(grids.size_of(SessionId(1)), (30, 100));
    assert_eq!(grids.size_of(SessionId(2)), (30, 100));
    assert_eq!(grids.size_of(SessionId(3)), (10, 40));
    assert_eq!(grids.shown(), [SessionId(2), SessionId(3)]);
}
