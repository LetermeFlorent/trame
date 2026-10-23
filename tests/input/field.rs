use trame::input::{MAX_CHARS, TextField};

fn typed(text: &str) -> TextField {
    let mut field = TextField::default();
    field.insert(text);
    field
}

#[test]
fn a_new_field_puts_the_caret_at_the_end() {
    let field = TextField::new("shell");
    assert_eq!(field.text(), "shell");
    assert_eq!(field.caret(), 5);
}

#[test]
fn insertion_happens_at_the_caret() {
    let mut field = typed("ac");
    field.left();
    field.insert("b");
    assert_eq!(field.text(), "abc");
    assert_eq!(field.before_caret(), "ab");
}

#[test]
fn an_empty_field_ignores_every_edit() {
    let mut field = TextField::default();
    field.backspace();
    field.delete();
    field.left();
    field.right();
    field.home();
    field.end();
    assert_eq!(field.text(), "");
    assert_eq!(field.caret(), 0);
}

#[test]
fn backspace_at_the_start_and_delete_at_the_end_do_nothing() {
    let mut field = typed("ab");
    field.delete();
    assert_eq!(field.text(), "ab");
    field.home();
    field.backspace();
    assert_eq!(field.text(), "ab");
    assert_eq!(field.caret(), 0);
}

#[test]
fn backspace_and_delete_remove_one_character_each_side() {
    let mut field = typed("abcd");
    field.left();
    field.left();
    field.backspace();
    assert_eq!(field.text(), "acd");
    field.delete();
    assert_eq!(field.text(), "ad");
    assert_eq!(field.caret(), 1);
}

#[test]
fn the_caret_stops_at_both_edges() {
    let mut field = typed("ab");
    field.right();
    assert_eq!(field.caret(), 2);
    field.home();
    field.left();
    assert_eq!(field.caret(), 0);
    field.end();
    assert_eq!(field.caret(), 2);
}

#[test]
fn multibyte_characters_move_and_delete_whole() {
    let mut field = typed("caf\u{e9}");
    assert_eq!(field.caret(), 5);
    field.left();
    assert_eq!(field.caret(), 3);
    field.right();
    field.backspace();
    assert_eq!(field.text(), "caf");
    field.insert("\u{e9}\u{e9}");
    field.left();
    field.delete();
    assert_eq!(field.text(), "caf\u{e9}");
}

#[test]
fn emoji_input_stays_on_character_boundaries() {
    let mut field = typed("a\u{1f600}b");
    field.left();
    assert_eq!(field.before_caret(), "a\u{1f600}");
    field.left();
    assert_eq!(field.before_caret(), "a");
    field.delete();
    assert_eq!(field.text(), "ab");
    field.end();
    field.insert("\u{1f680}");
    field.backspace();
    assert_eq!(field.text(), "ab");
}

#[test]
fn control_characters_are_dropped() {
    let field = typed("a\nb\tc\r");
    assert_eq!(field.text(), "abc");
    assert_eq!(TextField::new("x\u{7}y").text(), "xy");
}

#[test]
fn the_length_is_capped_in_characters() {
    let mut field = typed(&"\u{e9}".repeat(MAX_CHARS));
    field.insert("more");
    assert_eq!(field.text().chars().count(), MAX_CHARS);
    let long = TextField::new(&"x".repeat(MAX_CHARS + 20));
    assert_eq!(long.text().chars().count(), MAX_CHARS);
}

#[test]
fn inserting_an_empty_string_changes_nothing() {
    let mut field = typed("ab");
    field.insert("");
    assert_eq!(field, typed("ab"));
}

#[test]
fn a_masked_field_hides_each_character_and_keeps_the_caret_in_place() {
    let mut field = typed("pé5s");
    field.left();
    let masked = field.masked('*');
    assert_eq!(masked.text(), "****");
    assert_eq!(masked.before_caret(), "***");
    let bullets = field.masked('\u{2022}');
    assert_eq!(bullets.text().chars().count(), 4);
    assert_eq!(bullets.before_caret().chars().count(), 3);
    assert_eq!(TextField::default().masked('*').text(), "");
}
