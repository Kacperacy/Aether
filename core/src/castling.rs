use crate::File;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

impl CastlingRights {
    pub const EMPTY: CastlingRights = CastlingRights {
        short: None,
        long: None,
    };

    pub const fn is_empty(&self) -> bool {
        self.short.is_none() && self.long.is_none()
    }
}
