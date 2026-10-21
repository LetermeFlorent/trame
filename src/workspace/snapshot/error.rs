use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SnapshotError {
    Json(serde_json::Error),
    Version(Option<u64>),
    Empty,
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotError::Json(error) => write!(formatter, "invalid session file: {error}"),
            SnapshotError::Version(Some(version)) => {
                write!(formatter, "unsupported session file version {version}")
            }
            SnapshotError::Version(None) => write!(formatter, "session file has no version"),
            SnapshotError::Empty => write!(formatter, "session file holds no tab"),
        }
    }
}

impl Error for SnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SnapshotError::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for SnapshotError {
    fn from(error: serde_json::Error) -> Self {
        SnapshotError::Json(error)
    }
}
