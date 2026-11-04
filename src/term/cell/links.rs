pub const MAX_LINKS: usize = 4096;
pub const MAX_LINK_LEN: usize = 2048;

#[derive(Debug, Clone, Default)]
pub struct Links {
    uris: Vec<String>,
}

impl Links {
    pub fn get(&self, id: u16) -> Option<&str> {
        let index = usize::from(id).checked_sub(1)?;
        self.uris.get(index).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.uris.len()
    }

    pub fn is_empty(&self) -> bool {
        self.uris.is_empty()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.uris.len() >= MAX_LINKS
    }

    pub(crate) fn intern(&mut self, uri: String) -> Option<u16> {
        if let Some(index) = self.uris.iter().rposition(|known| *known == uri) {
            return u16::try_from(index + 1).ok();
        }
        if self.is_full() {
            return None;
        }
        self.uris.push(uri);
        u16::try_from(self.uris.len()).ok()
    }

    pub(crate) fn keep(&mut self, used: &[bool]) -> Vec<u16> {
        let mut map = vec![0; self.uris.len() + 1];
        let mut kept = Vec::new();
        for (index, uri) in std::mem::take(&mut self.uris).into_iter().enumerate() {
            if used.get(index + 1).copied().unwrap_or(false) {
                kept.push(uri);
                map[index + 1] = u16::try_from(kept.len()).unwrap_or(0);
            }
        }
        self.uris = kept;
        map
    }
}
