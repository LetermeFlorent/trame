pub const MAX_PARAMS: usize = 32;

#[derive(Debug, Clone)]
pub struct Params {
    values: [u16; MAX_PARAMS],
    group_lens: [u8; MAX_PARAMS],
    len: usize,
    group_start: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            values: [0; MAX_PARAMS],
            group_lens: [0; MAX_PARAMS],
            len: 0,
            group_start: 0,
        }
    }
}

impl Params {
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_full(&self) -> bool {
        self.len == MAX_PARAMS
    }

    pub fn iter(&self) -> ParamsIter<'_> {
        ParamsIter {
            params: self,
            index: 0,
        }
    }

    pub fn group(&self, index: usize) -> Option<&[u16]> {
        self.iter().nth(index)
    }

    pub fn first_or(&self, index: usize, fallback: u16) -> u16 {
        match self.group(index).and_then(|group| group.first()) {
            Some(&0) | None => fallback,
            Some(&value) => value,
        }
    }

    pub fn raw(&self, index: usize) -> u16 {
        self.group(index)
            .and_then(|group| group.first().copied())
            .unwrap_or(0)
    }

    pub(super) fn clear(&mut self) {
        self.len = 0;
        self.group_start = 0;
    }

    pub(super) fn push(&mut self, value: u16) {
        self.group_start = self.len;
        self.group_lens[self.len] = 1;
        self.values[self.len] = value;
        self.len += 1;
    }

    pub(super) fn extend(&mut self, value: u16) {
        self.group_lens[self.group_start] += 1;
        self.values[self.len] = value;
        self.len += 1;
    }
}

pub struct ParamsIter<'a> {
    params: &'a Params,
    index: usize,
}

impl<'a> Iterator for ParamsIter<'a> {
    type Item = &'a [u16];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.params.len {
            return None;
        }
        let group_len = usize::from(self.params.group_lens[self.index]);
        let group = &self.params.values[self.index..self.index + group_len];
        self.index += group_len;
        Some(group)
    }
}
