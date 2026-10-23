use std::fmt;

use super::RELEASES;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    pub fn current() -> Option<Self> {
        Self::parse(env!("CARGO_PKG_VERSION"))
    }

    pub fn parse(text: &str) -> Option<Self> {
        let text = text.strip_prefix('v').unwrap_or(text);
        let mut parts = text.split('.').map(number);
        let version = Self {
            major: parts.next()??,
            minor: parts.next()??,
            patch: parts.next()??,
        };
        parts.next().is_none().then_some(version)
    }

    pub fn number(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn released(location: &str) -> Option<Self> {
        let tag = location
            .trim()
            .strip_prefix(RELEASES)?
            .strip_prefix("/tag/")?;
        tag.starts_with('v').then(|| Self::parse(tag)).flatten()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "v{}", self.number())
    }
}

fn number(part: &str) -> Option<u64> {
    let digits = !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    digits.then(|| part.parse().ok()).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(text: &str) -> Version {
        Version::parse(text).expect(text)
    }

    #[test]
    fn a_tag_reads_with_or_without_its_v() {
        assert_eq!(version("v1.2.3"), version("1.2.3"));
        assert_eq!(version("v0.10.7").to_string(), "v0.10.7");
        assert_eq!(version("v0.10.7").number(), "0.10.7");
    }

    #[test]
    fn anything_but_three_plain_numbers_is_refused() {
        for text in [
            "",
            "v",
            "v1.2",
            "v1.2.3.4",
            "v1.2.3-rc1",
            "v1..3",
            "va.b.c",
            "v1.2.+3",
            "vv1.2.3",
        ] {
            assert_eq!(Version::parse(text), None, "{text}");
        }
    }

    #[test]
    fn numbers_compare_as_numbers_not_as_text() {
        assert!(version("0.10.0") > version("0.9.9"));
        assert!(version("1.0.0") > version("0.99.99"));
        assert!(version("0.2.1") > version("0.2.0"));
        assert_eq!(
            version("0.2.0").cmp(&version("v0.2.0")),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn the_running_version_is_readable() {
        assert!(Version::current().is_some());
    }

    #[test]
    fn the_latest_release_comes_from_the_redirect() {
        let location = format!("{RELEASES}/tag/v0.3.1\n");
        assert_eq!(Version::released(&location), Some(version("0.3.1")));
    }

    #[test]
    fn a_repository_without_release_or_another_page_gives_nothing() {
        for location in [
            String::new(),
            RELEASES.to_owned(),
            format!("{RELEASES}/tag/0.3.1"),
            format!("{RELEASES}/tag/v0.3.1-beta"),
            "https://github.com/someone/else/releases/tag/v9.0.0".to_owned(),
            "https://github.com/login".to_owned(),
        ] {
            assert_eq!(Version::released(&location), None, "{location}");
        }
    }
}
