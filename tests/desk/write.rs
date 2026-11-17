use trame::desk::Effect;

use crate::tabs::opened;

#[test]
fn writing_nothing_changes_nothing() {
    let (mut desk, _) = opened();
    assert!(desk.write_to_active(|_| Vec::new()).is_empty());
}

#[test]
fn writing_bytes_reports_the_typed_tab() {
    let (mut desk, id) = opened();
    let effects = desk.write_to_active(|_| b"true\r".to_vec());
    assert_eq!(effects.as_slice(), [Effect::Typed(id)]);
}
