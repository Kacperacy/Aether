use crate::File;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

impl Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "-")
        } else {
            if self.short.is_some() {
                write!(f, "K")?;
            }
            if self.long.is_some() {
                write!(f, "Q")?;
            }
            Ok(())
        }
    }
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
