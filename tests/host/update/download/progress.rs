use std::cell::RefCell;

use trame::host::update::download::progress::follow;

fn percents(output: &str) -> Vec<u8> {
    let seen = RefCell::new(Vec::new());
    follow(output.as_bytes(), &|percent| {
        seen.borrow_mut().push(percent)
    });
    seen.into_inner()
}

#[test]
fn each_new_whole_percent_of_the_curl_bar_is_reported_once() {
    let output = "\r#=#=#\r##O#-#\r###  15.7%\r###  15.9%\r######  62.9%\r########  100.0%\n";
    assert_eq!(percents(output), [15, 62, 100]);
}

#[test]
fn a_bar_without_a_size_or_an_error_reports_nothing() {
    assert!(
        percents("\r#=#=#\r##O#-#\ncurl: (22) The requested URL returned error: 404\n").is_empty()
    );
}
