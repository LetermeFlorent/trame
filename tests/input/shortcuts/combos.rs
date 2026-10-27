use trame::input::Combo;

#[test]
fn writes_back_the_names_it_reads() {
    let names = [
        "Ctrl+Shift+T",
        "Alt+1",
        "Ctrl++",
        "Ctrl+-",
        "Ctrl+Num0",
        "Ctrl+Num+",
        "Shift+PageUp",
        "F5",
        "Ctrl+Alt+Left",
        "Shift+Super+D",
    ];
    for name in names {
        assert_eq!(
            Combo::parse(name).map(|combo| combo.text()),
            Some(name.into())
        );
    }
    let loose = Combo::parse(" ctrl + shift + t ").map(|combo| combo.text());
    assert_eq!(loose, Some("Ctrl+Shift+T".into()));
    assert_eq!(Combo::parse("Cmd+T"), Combo::parse("Super+T"));
}

#[test]
fn refuses_names_it_does_not_know() {
    for name in [
        "",
        "Ctrl+",
        "Ctrl+Foo",
        "Hyper+T",
        "Ctrl+Ctrl+T",
        "Ctrl+MediaPlay",
    ] {
        assert_eq!(Combo::parse(name), None, "{name}");
    }
}
