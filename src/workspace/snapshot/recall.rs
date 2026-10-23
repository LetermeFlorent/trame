use super::{InstanceSnapshot, NodeSnapshot, PaneSnapshot, Snapshot, TabSnapshot};
use crate::preferences::Recall;

impl Snapshot {
    pub fn recalled(&self) -> Option<Snapshot> {
        if self.resume {
            return Some(self.clone());
        }
        let preferences = &self.preferences;
        if !preferences.recalls(Recall::Instances) {
            return None;
        }
        if preferences.recalls(Recall::Tabs) {
            return Some(self.clone());
        }
        let count = usize::from(preferences.instance_tabs);
        let instances = self
            .instances
            .iter()
            .map(|instance| {
                let tab = TabSnapshot {
                    cwd: instance
                        .folder
                        .clone()
                        .or_else(|| preferences.default_folder.clone()),
                    ..TabSnapshot::default()
                };
                InstanceSnapshot {
                    active_pane: 0,
                    root: NodeSnapshot::Pane(PaneSnapshot {
                        tabs: vec![tab; count],
                        active: 0,
                    }),
                    ..instance.clone()
                }
            })
            .collect();
        Some(Snapshot {
            instances,
            ..self.clone()
        })
    }
}
