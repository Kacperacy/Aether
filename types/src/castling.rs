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
}
