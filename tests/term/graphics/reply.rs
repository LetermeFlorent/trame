use trame::term::graphics::{Error, reply, wanted};

#[test]
fn formats_ok_replies() {
    assert_eq!(reply(31, 0, 0, Ok(())), b"\x1b_Gi=31;OK\x1b\\");
    assert_eq!(reply(5, 9, 2, Ok(())), b"\x1b_Gi=5,I=9,p=2;OK\x1b\\");
}

#[test]
fn formats_error_replies() {
    assert_eq!(
        reply(7, 0, 0, Err(Error::NoEnt)),
        b"\x1b_Gi=7;ENOENT:no such image\x1b\\"
    );
    assert_eq!(
        reply(7, 3, 4, Err(Error::BadF)),
        b"\x1b_Gi=7,I=3,p=4;EBADF:unsupported transport\x1b\\"
    );
    assert_eq!(Error::Inval.code(), "EINVAL:invalid");
    assert_eq!(Error::TooBig.code(), "EFBIG:too large");
}

#[test]
fn quiet_levels_filter_replies() {
    assert!(wanted(0, true) && wanted(0, false));
    assert!(!wanted(1, true) && wanted(1, false));
    assert!(!wanted(2, true) && !wanted(2, false));
}
