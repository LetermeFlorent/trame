use std::fmt;

#[derive(Debug)]
pub enum ReadbackError {
    Poll(wgpu::PollError),
    Map(wgpu::BufferAsyncError),
    Range(wgpu::MapRangeError),
}

impl fmt::Display for ReadbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadbackError::Poll(error) => write!(formatter, "the graphics device stalled: {error}"),
            ReadbackError::Map(error) => {
                write!(formatter, "the pixels could not be mapped: {error}")
            }
            ReadbackError::Range(error) => {
                write!(formatter, "the pixels could not be read: {error}")
            }
        }
    }
}

impl std::error::Error for ReadbackError {}
