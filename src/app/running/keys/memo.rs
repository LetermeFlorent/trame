pub(in crate::app::running) struct Memo<K, V> {
    entry: Option<(K, V)>,
}

impl<K, V> Default for Memo<K, V> {
    fn default() -> Self {
        Self { entry: None }
    }
}

impl<K: Clone + PartialEq, V> Memo<K, V> {
    pub(in crate::app::running) fn get(&mut self, key: &K, build: impl FnOnce(&K) -> V) -> &V {
        if self.entry.as_ref().is_some_and(|(kept, _)| kept != key) {
            self.entry = None;
        }
        let (_, value) = self.entry.get_or_insert_with(|| (key.clone(), build(key)));
        value
    }
}
