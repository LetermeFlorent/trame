use trame::theme::{hex_code, parse_hex};

#[test]
fn a_hex_code_reads_with_or_without_its_hash_and_in_short_form() {
    assert_eq!(parse_hex("#12ab34"), Some(0x12ab34));
    assert_eq!(parse_hex("12AB34"), Some(0x12ab34));
    assert_eq!(parse_hex("#fa0"), Some(0xffaa00));
    assert_eq!(parse_hex("#12ab3"), None);
    assert_eq!(parse_hex("#12ab3g"), None);
    assert_eq!(hex_code(0x12ab34), "#12AB34");
}
