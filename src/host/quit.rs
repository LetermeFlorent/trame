#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restart {
    Update,
    Again,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quit {
    Close,
    Restart(Restart),
}
