use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::SnapshotError;
use crate::preferences::{Preferences, SshHost};
use crate::theme::{Emblem, Swatch};
use crate::workspace::Axis;

pub const SNAPSHOT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub sidebar_open: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sidebar_width: Option<f32>,
    #[serde(flatten)]
    pub preferences: Preferences,
    pub active_instance: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowSnapshot>,
    pub instances: Vec<InstanceSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowSnapshot {
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstanceSnapshot {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Swatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emblem: Option<Emblem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<PathBuf>,
    pub active_pane: usize,
    pub root: NodeSnapshot,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeSnapshot {
    Pane(PaneSnapshot),
    Split {
        axis: Axis,
        ratios: Vec<f32>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pinned: Vec<bool>,
        children: Vec<NodeSnapshot>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaneSnapshot {
    pub tabs: Vec<TabSnapshot>,
    pub active: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Swatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshHost>,
}

impl Snapshot {
    pub fn from_json(text: &str) -> Result<Self, SnapshotError> {
        let value: Value = serde_json::from_str(text)?;
        let version = value.get("version").and_then(Value::as_u64);
        if version != Some(u64::from(SNAPSHOT_VERSION)) {
            return Err(SnapshotError::Version(version));
        }
        Ok(serde_json::from_value(value)?)
    }

    pub fn to_json(&self) -> Result<String, SnapshotError> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
