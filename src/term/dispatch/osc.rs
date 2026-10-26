use super::decode::{decode_base64, file_url_path};
use crate::term::screen::Screen;

const MAX_TITLE_LEN: usize = 256;

impl Screen {
    pub(crate) fn osc(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let Some(kind) = params.first() else {
            return;
        };
        match *kind {
            b"0" | b"2" => self.set_title(&params[1..]),
            b"7" => {
                if let Some(path) = params.get(1).and_then(|url| file_url_path(url)) {
                    self.cwd = Some(path);
                }
            }
            b"9" if params.get(1) == Some(&&b"9"[..]) => {
                if let Some(path) = params.get(2) {
                    let path = String::from_utf8_lossy(path);
                    self.cwd = Some(path.trim_matches('"').to_string());
                }
                // The shell only sends this from its prompt, so whatever asked for mouse
                // reports is gone, maybe without turning them off (crash, dropped SSH).
                self.forget_mouse();
            }
            b"9" if !params.get(1).is_some_and(|code| is_number(code)) => {
                self.notice(&params[1..]);
            }
            b"777" if params.get(1) == Some(&&b"notify"[..]) => {
                self.notice(params.get(3..).unwrap_or_default());
            }
            b"99" => self.notice(params.get(2..).unwrap_or_default()),
            b"10" | b"11" | b"12" if params.get(1) == Some(&&b"?"[..]) => {
                self.report_color(kind, bell_terminated);
            }
            b"52" => {
                let data = params.get(2).copied().unwrap_or_default();
                if data != b"?" {
                    let decoded = decode_base64(data)
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                        .filter(|text| !text.is_empty());
                    self.clipboard = decoded.or(self.clipboard.take());
                }
            }
            _ => {}
        }
    }

    fn set_title(&mut self, parts: &[&[u8]]) {
        let joined = parts.join(&b';');
        let mut title: String = String::from_utf8_lossy(&joined)
            .chars()
            .filter(|c| !c.is_control())
            .take(MAX_TITLE_LEN)
            .collect();
        title = title.trim().to_string();
        self.title = if title.is_empty() { None } else { Some(title) };
    }

    fn notice(&mut self, parts: &[&[u8]]) {
        let joined = parts.join(&b';');
        let text: String = String::from_utf8_lossy(&joined)
            .chars()
            .filter(|c| !c.is_control())
            .take(MAX_TITLE_LEN)
            .collect();
        self.attention = Some(text.trim().to_string());
    }

    fn report_color(&mut self, kind: &[u8], bell_terminated: bool) {
        let rgb = match kind {
            b"10" => self.colors.foreground,
            b"11" => self.colors.background,
            _ => self.colors.cursor,
        };
        let terminator = if bell_terminated { "\x07" } else { "\x1b\\" };
        let code = String::from_utf8_lossy(kind);
        let reply = format!(
            "\x1b]{code};rgb:{0:02x}{0:02x}/{1:02x}{1:02x}/{2:02x}{2:02x}{terminator}",
            rgb[0], rgb[1], rgb[2]
        );
        self.respond(reply.as_bytes());
    }
}

fn is_number(code: &[u8]) -> bool {
    !code.is_empty() && code.iter().all(u8::is_ascii_digit)
}
