//! List items shared by the panes and the action modals.

#[derive(Debug, Clone)]
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) starred: bool,
    /// True if this version is the active `current` symlink target.
    pub(crate) is_current: bool,
    /// True if this version is the `default` symlink target.
    pub(crate) is_default: bool,
}

impl Item {
    pub(crate) fn new(name: impl Into<String>, starred: bool) -> Self {
        Self {
            name: name.into(),
            starred,
            is_current: false,
            is_default: false,
        }
    }
}

/// Sort items: installed first (default → current → other installed, each by
/// version descending), then uninstalled (version descending).
pub(crate) fn sort_items(items: &mut [Item]) {
    items.sort_by(|a, b| {
        // Installed (starred) bubble to top.
        b.starred
            .cmp(&a.starred)
            // Within installed: default first, then current, then others.
            .then_with(|| {
                let rank = |i: &Item| match (i.is_default, i.is_current) {
                    (true, _) => 0,
                    (false, true) => 1,
                    (false, false) => 2,
                };
                rank(a).cmp(&rank(b))
            })
            // Within the same rank: version descending (latest first).
            .then_with(|| b.name.cmp(&a.name))
    });
}

pub(crate) fn filter_items(items: &[Item], query: &str) -> Vec<Item> {
    if query.is_empty() {
        return items.to_vec();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .filter(|i| i.name.to_lowercase().contains(&q))
        .cloned()
        .collect()
}
