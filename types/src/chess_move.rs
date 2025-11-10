use crate::{CastlingRights, Color, Piece, Square};
use std::fmt::{self, Display, Formatter};

/// Represents a chess move.
///
/// A move includes the origin and destination squares, the piece being moved,
/// optional capture and promotion information, and special move flags.
///
/// # Examples
///
/// ```
/// use aether_types::{Move, Square, Piece};
///
/// let mv = Move::new(Square::E2, Square::E4)
///     .with_piece(Piece::Pawn);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    /// Origin square of the move
    pub from: Square,
    /// Destination square of the move
    pub to: Square,
    /// The piece being moved
    pub piece: Piece,
    /// The piece being captured, if any
    pub capture: Option<Piece>,
    /// The piece to promote to (for pawn promotions)
    pub promotion: Option<Piece>,
    /// Special move flags (castling, en passant, double pawn push)
    pub flags: MoveFlags,
}

/// Flags indicating special move types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveFlags {
    /// True if this move is a castling move
    pub is_castle: bool,
    /// True if this move is an en passant capture
    pub is_en_passant: bool,
    /// True if this move is a double pawn push (pawn moves two squares)
    pub is_double_pawn_push: bool,
}

/// State information needed to unmake a move.
///
/// This stores all the information necessary to restore the board to its
/// previous state after making a move, including captured pieces, castling
/// rights, en passant square, halfmove clock, and zobrist hash.
#[derive(Debug, Clone)]
pub struct MoveState {
    /// The piece that was captured, if any, along with its color
    pub captured_piece: Option<(Piece, Color)>,
    /// Origin square of the move
    pub mv_from: Square,
    /// Destination square of the move
    pub mv_to: Square,
    /// The piece promoted to, if any
    pub promotion: Option<Piece>,

    /* game-state members */
    /// The zobrist hash before the move
    pub old_zobrist_hash: u64,
    /// The en passant square before the move
    pub old_en_passant: Option<Square>,
    /// The castling rights before the move [White, Black]
    pub old_castling_rights: [CastlingRights; 2],
    /// The halfmove clock before the move (for fifty-move rule)
    pub old_halfmove_clock: u16,
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 1. origin + destination squares
        write!(f, "{}{}", self.from, self.to)?;

        // 2. optional promotion suffix (UCI uses lowercase piece letters)
        if let Some(p) = self.promotion {
            let symbol = match p {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                // King and Pawn can never appear here – unreachable by rule
                _ => unreachable!("illegal promotion piece"),
            };
            write!(f, "{symbol}")?;
        }

        Ok(())
    }
}

impl Move {
    /// Creates a new move from origin to destination square.
    ///
    /// Note: This constructor defaults to `Piece::Pawn`. Use `.with_piece()`
    /// to set the correct piece type.
    pub fn new(from: Square, to: Square) -> Self {
        Move {
            from,
            to,
            piece: Piece::Pawn, // This should be properly determined
            capture: None,
            promotion: None,
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: false,
            },
        }
    }

    /// Builder method to set the promotion piece (for pawn promotions).
    pub fn with_promotion(mut self, piece: Piece) -> Self {
        self.promotion = Some(piece);
        self
    }

    /// Builder method to set the piece being moved.
    pub fn with_piece(mut self, piece: Piece) -> Self {
        self.piece = piece;
        self
    }

    /// Builder method to set the captured piece.
    pub fn with_capture(mut self, piece: Piece) -> Self {
        self.capture = Some(piece);
        self
    }

    /// Builder method to set move flags (castling, en passant, etc.).
    pub fn with_flags(mut self, flags: MoveFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Returns `true` if this move captures a piece.
    pub fn is_capture(&self) -> bool {
        self.capture.is_some()
    }
}
