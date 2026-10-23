use trame::input::{MAX_CHARS, TextField};

fn selected(text: &str) -> TextField {
    let mut field = TextField::new(text);
    field.select_all();
    field
}

#[test]
fn selecting_all_keeps_the_text_and_parks_the_caret_at_the_end() {
    let field = selected("shell");
    assert!(field.is_selected());
    assert_eq!(field.text(), "shell");
    assert_eq!(field.caret(), 5);
}

#[test]
fn an_empty_field_has_nothing_to_select() {
    let mut field = TextField::default();
    field.select_all();
    assert!(!field.is_selected());
}

#[test]
fn typing_over_a_selection_replaces_the_whole_text() {
    let mut field = selected("Terminal 2");
    field.insert("w");
    assert_eq!(field.text(), "w");
    assert_eq!(field.caret(), 1);
    assert!(!field.is_selected());
    field.insert("eb");
    assert_eq!(field.text(), "web");
}

#[test]
fn backspace_or_delete_on_a_selection_empties_the_field() {
    for erase in [TextField::backspace, TextField::delete] {
        let mut field = selected("logs");
        erase(&mut field);
        assert_eq!(field.text(), "");
        assert_eq!(field.caret(), 0);
        assert!(!field.is_selected());
    }
}

#[test]
fn arrows_drop_the_selection_at_its_edges() {
    let mut field = selected("logs");
    field.left();
    assert_eq!((field.text(), field.caret()), ("logs", 0));
    assert!(!field.is_selected());
    let mut field = selected("logs");
    field.right();
    assert_eq!((field.text(), field.caret()), ("logs", 4));
    assert!(!field.is_selected());
}

#[test]
fn home_and_end_drop_the_selection_without_erasing() {
    let mut field = selected("logs");
    field.home();
    assert!(!field.is_selected());
    field.insert("x");
    assert_eq!(field.text(), "xlogs");
    let mut field = selected("logs");
    field.end();
    field.insert("!");
    assert_eq!(field.text(), "logs!");
}

#[test]
fn control_characters_leave_the_selection_in_place() {
    let mut field = selected("logs");
    field.insert("\r\t");
    assert_eq!(field.text(), "logs");
    assert!(field.is_selected());
}

#[test]
fn a_full_field_still_takes_a_replacement_over_its_selection() {
    let mut field = selected(&"a".repeat(MAX_CHARS));
    field.insert(&"b".repeat(MAX_CHARS + 5));
    assert_eq!(field.text(), "b".repeat(MAX_CHARS));
}

#[test]
fn a_masked_field_keeps_the_selection() {
    let masked = selected("secret").masked('*');
    assert!(masked.is_selected());
    assert_eq!(masked.text(), "******");
}
