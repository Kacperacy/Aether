use crate::Board;
use aether_core::{BitBoard, Color, Piece, Square};

pub trait BoardQuery {
    /// Piece and color at square, if any
    fn piece_at(&self, square: Square) -> Option<(Piece, Color)>;
    /// True if any piece occupies the square
    fn is_square_occupied(&self, square: Square) -> bool;
    /// True if the given square is attacked by pieces of `by_color` under the current position
    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool;
    /// Count of `piece` for `color`
    fn piece_count(&self, piece: Piece, color: Color) -> u32;
    /// King square for `color`, if present
    fn get_king_square(&self, color: Color) -> Option<Square>;
    /// BitBoard of all squares occupied by `color`
    fn occupied_by(&self, color: Color) -> BitBoard;
    /// BitBoard of all occupied squares
    fn occupied(&self) -> BitBoard;
    /// BitBoard of all squares occupied by `piece` of `color`
    fn piece_bb(&self, piece: Piece, color: Color) -> BitBoard;
    /// Returns reference to piece bitboards [color][piece]
    fn pieces(&self) -> &[[BitBoard; 6]; 2];
    /// Whether side can castle short (right exists); path safety is validated by consumers
    fn can_castle_short(&self, color: Color) -> bool;
    /// Whether side can castle long (right exists); path safety is validated by consumers
    fn can_castle_long(&self, color: Color) -> bool;
    /// En-passant target square, if any
    fn en_passant_square(&self) -> Option<Square>;
    /// Side to move
    fn side_to_move(&self) -> Color;
    /// Returns the Zobrist hash of the current position
    fn zobrist_hash_raw(&self) -> u64;
    /// Checks for insufficient material to continue the game
    fn is_insufficient_material(&self) -> bool;
    /// Checks for threefold repetition
    fn is_threefold_repetition(&self) -> bool;
    /// Checks for twofold repetition
    fn is_twofold_repetition(&self) -> bool;
    /// Checks for fifty-move rule draw
    fn is_fifty_move_draw(&self) -> bool;
    /// Checks for all draw conditions
    fn is_draw(&self) -> bool;
}

impl BoardQuery for Board {
    fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.mailbox[square.to_index() as usize]
    }

    fn is_square_occupied(&self, square: Square) -> bool {
        self.cache.occupied.has(square)
    }

    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        !self.attackers_to_square(square, by_color).is_empty()
    }

    #[inline(always)]
    fn piece_count(&self, piece: Piece, color: Color) -> u32 {
        self.pieces[color as usize][piece as usize].len()
    }

    #[inline(always)]
    fn get_king_square(&self, color: Color) -> Option<Square> {
        // Use bitboard lookup (cache version had issues)
        self.pieces[color as usize][Piece::King as usize].to_square()
    }

    #[inline(always)]
    fn occupied_by(&self, color: Color) -> BitBoard {
        self.cache.color_combined[color as usize]
    }

    #[inline(always)]
    fn occupied(&self) -> BitBoard {
        self.cache.occupied
    }

    #[inline(always)]
    fn piece_bb(&self, piece: Piece, color: Color) -> BitBoard {
        self.pieces[color as usize][piece as usize]
    }

    #[inline(always)]
    fn pieces(&self) -> &[[BitBoard; 6]; 2] {
        &self.pieces
    }

    fn can_castle_short(&self, color: Color) -> bool {
        self.game_state.castling_rights[color as usize]
            .short
            .is_some()
    }

    fn can_castle_long(&self, color: Color) -> bool {
        self.game_state.castling_rights[color as usize]
            .long
            .is_some()
    }

    fn en_passant_square(&self) -> Option<Square> {
        self.game_state.en_passant_square
    }

    fn side_to_move(&self) -> Color {
        self.game_state.side_to_move
    }

    fn zobrist_hash_raw(&self) -> u64 {
        self.zobrist_hash
    }

    fn is_insufficient_material(&self) -> bool {
        if self.piece_count(Piece::Pawn, Color::White) > 0
            || self.piece_count(Piece::Pawn, Color::Black) > 0
        {
            return false;
        }

        if self.piece_count(Piece::Rook, Color::White) > 0
            || self.piece_count(Piece::Rook, Color::Black) > 0
        {
            return false;
        }

        if self.piece_count(Piece::Queen, Color::White) > 0
            || self.piece_count(Piece::Queen, Color::Black) > 0
        {
            return false;
        }

        let white_knights = self.piece_count(Piece::Knight, Color::White);
        let black_knights = self.piece_count(Piece::Knight, Color::Black);
        let white_bishops = self.piece_count(Piece::Bishop, Color::White);
        let black_bishops = self.piece_count(Piece::Bishop, Color::Black);

        // Now we only have kings, bishops, and knights
        let white_minors = white_bishops + white_knights;
        let black_minors = black_bishops + black_knights;

        // K vs K
        if white_minors == 0 && black_minors == 0 {
            return true;
        }

        // K+B vs K or K+N vs K
        if white_minors == 1 && black_minors == 0 {
            return true;
        }
        if white_minors == 0 && black_minors == 1 {
            return true;
        }

        // K+B vs K+B on same color squares
        if white_bishops == 1 && black_bishops == 1 && white_knights == 0 && black_knights == 0 {
            return self.are_bishops_on_same_color();
        }

        // All other cases have sufficient material
        false
    }

    /// Returns true if the current position has occurred 3+ times in the game.
    ///
    /// Note: `repetition_count()` returns the number of PREVIOUS occurrences
    /// (not including the current position). So:
    /// - repetition_count() == 0: position seen for the 1st time
    /// - repetition_count() == 1: position seen for the 2nd time (twofold)
    /// - repetition_count() >= 2: position seen for the 3rd+ time (threefold)
    #[inline]
    fn is_threefold_repetition(&self) -> bool {
        self.repetition_count() >= 2
    }

    /// Returns true if the current position has occurred 2+ times in the game.
    ///
    /// Useful for search to detect potential draws before they occur.
    /// See `is_threefold_repetition()` for semantics of `repetition_count()`.
    #[inline]
    fn is_twofold_repetition(&self) -> bool {
        self.repetition_count() >= 1
    }

    #[inline]
    fn is_fifty_move_draw(&self) -> bool {
        self.game_state.halfmove_clock >= 100
    }

    #[inline]
    fn is_draw(&self) -> bool {
        self.is_fifty_move_draw()
            || self.is_threefold_repetition()
            || self.is_insufficient_material()
    }
}

impl Board {
    fn are_bishops_on_same_color(&self) -> bool {
        // Get bishop squares
        let white_bishop_bb = &self.pieces[Color::White as usize][Piece::Bishop as usize];
        let black_bishop_bb = &self.pieces[Color::Black as usize][Piece::Bishop as usize];

        let Some(white_sq) = white_bishop_bb.to_square() else {
            return false; // Should not happen
        };

        let Some(black_sq) = black_bishop_bb.to_square() else {
            return false; // Should not happen
        };

        let white_is_light = (white_sq.file() as usize + white_sq.rank() as usize) % 2 == 0;
        let black_is_light = (black_sq.file() as usize + black_sq.rank() as usize) % 2 == 0;

        white_is_light == black_is_light
    }
}
