use trame::term::graphics::{Action, Command, Crop, Format, parse, split};

#[test]
fn empty_control_uses_kitty_defaults() {
    let command = parse(b"");
    assert_eq!(command, Command::default());
    assert_eq!(command.action, Action::Transmit);
    assert_eq!(command.format, Format::Rgba);
    assert_eq!(command.transport, b'd');
    assert!(!command.more && !command.compressed);
}

#[test]
fn reads_the_probe_keys() {
    let command = parse(b"i=31,s=1,v=1,a=q,t=d,f=24");
    assert_eq!(command.id, 31);
    assert_eq!((command.width, command.height), (1, 1));
    assert_eq!(command.action, Action::Query);
    assert_eq!(command.transport, b'd');
    assert_eq!(command.format, Format::Rgb);
}

#[test]
fn reads_placement_keys() {
    let command = parse(b"a=T,I=7,p=3,c=10,r=4,x=1,y=2,w=3,h=4,X=5,Y=6,C=1,z=-9,U=1,o=z,m=1,q=2");
    assert_eq!(command.action, Action::TransmitPlace);
    assert_eq!((command.number, command.placement), (7, 3));
    assert_eq!((command.cols, command.rows), (10, 4));
    assert_eq!(
        command.crop,
        Crop {
            x: 1,
            y: 2,
            w: 3,
            h: 4
        }
    );
    assert_eq!(command.offset, (5, 6));
    assert!(command.cursor_stays && command.virtual_ && command.compressed && command.more);
    assert_eq!((command.z, command.quiet), (-9, 2));
}

#[test]
fn ignores_unknown_keys_and_bad_values() {
    let command = parse(b"K=9,i=4,zz=1,s=abc,=,f=100,a=x,S=12");
    assert_eq!(command.id, 4);
    assert_eq!(command.width, 0);
    assert_eq!(command.format, Format::Png);
    assert_eq!(command.action, Action::Unsupported);
}

#[test]
fn reads_delete_letter() {
    assert_eq!(parse(b"a=d,d=I,i=2").delete, b'I');
    assert_eq!(parse(b"a=d").delete, b'a');
}

#[test]
fn splits_control_from_payload() {
    assert_eq!(split(b"Ga=t;AAAA"), Some((&b"a=t"[..], &b"AAAA"[..])));
    assert_eq!(split(b"Ga=d"), Some((&b"a=d"[..], &b""[..])));
    assert_eq!(split(b"Gm=0;a;b"), Some((&b"m=0"[..], &b"a;b"[..])));
    assert_eq!(split(b"Xa=t;AAAA"), None);
    assert_eq!(split(b""), None);
}
