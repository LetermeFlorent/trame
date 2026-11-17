#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Inval,
    NoEnt,
    BadF,
    TooBig,
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Inval => "EINVAL:invalid",
            Self::NoEnt => "ENOENT:no such image",
            Self::BadF => "EBADF:unsupported transport",
            Self::TooBig => "EFBIG:too large",
        }
    }
}
