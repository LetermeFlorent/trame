use std::io::{BufRead, BufReader, Read};

const REDRAW: u8 = b'\r';
const PERCENT: char = '%';
const FULL: f32 = 100.0;

pub fn follow(stream: impl Read, on_percent: &dyn Fn(u8)) {
    let mut last = None;
    for update in BufReader::new(stream).split(REDRAW).map_while(Result::ok) {
        let Some(percent) = percent(&update).filter(|&percent| last != Some(percent)) else {
            continue;
        };
        last = Some(percent);
        on_percent(percent);
    }
}

fn percent(update: &[u8]) -> Option<u8> {
    let text = String::from_utf8_lossy(update);
    let number = text.split_whitespace().last()?.strip_suffix(PERCENT)?;
    let value: f32 = number.parse().ok()?;
    Some(value.clamp(0.0, FULL) as u8)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

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
            percents("\r#=#=#\r##O#-#\ncurl: (22) The requested URL returned error: 404\n")
                .is_empty()
        );
    }
}
