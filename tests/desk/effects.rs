use trame::desk::{Effect, Effects};

#[test]
fn pushed_and_appended_effects_keep_their_order() {
    let mut effects = Effects::default();
    assert!(effects.is_empty());
    effects.push(Effect::Relayout);
    let mut later = Effects::from(Effect::Animate);
    later.push(Effect::Changed);
    effects.append(later);
    effects.push(Effect::Raise);
    let expected = [
        Effect::Relayout,
        Effect::Animate,
        Effect::Changed,
        Effect::Raise,
    ];
    assert_eq!(effects.as_slice(), expected);
    assert_eq!(effects.into_iter().collect::<Vec<_>>(), expected);
}

#[test]
fn one_effect_becomes_a_list_of_one() {
    let effects = Effects::from(Effect::Redraw);
    assert!(!effects.is_empty());
    assert_eq!(effects.as_slice(), [Effect::Redraw]);
}
