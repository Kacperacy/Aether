use crate::Board;
use aether_core::{BitBoard, CastlingRights, Color, Piece, Square};

impl Board {
    #[inline(always)]
    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.mailbox[square.to_index() as usize]
    }

    #[inline(always)]
    pub fn is_square_occupied(&self, square: Square) -> bool {
        self.cache.occupied.contains(square)
    }

    /// Uses the short-circuiting attack test rather than building the full
    /// attacker set and checking emptiness.
    #[inline(always)]
    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        attacks::is_square_attacked(
            square,
            by_color,
            self.cache.occupied,
            &self.pieces[by_color as usize],
        )
    }

    #[inline(always)]
    pub fn piece_count(&self, piece: Piece, color: Color) -> usize {
        self.pieces[color as usize][piece as usize].count()
    }

    #[inline(always)]
    pub fn get_king_square(&self, color: Color) -> Square {
        self.state.king_square[color as usize]
    }

    #[inline(always)]
    pub fn occupied_by(&self, color: Color) -> BitBoard {
        self.cache.color_combined[color as usize]
    }

    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self.cache.occupied
    }

    #[inline(always)]
    pub fn piece_bb(&self, piece: Piece, color: Color) -> BitBoard {
        self.pieces[color as usize][piece as usize]
    }

    #[inline(always)]
    pub fn pieces(&self) -> &[[BitBoard; 6]; 2] {
        &self.pieces
    }

    #[inline]
    pub fn can_castle_short(&self, color: Color) -> bool {
        self.state
            .castling_rights
            .contains(CastlingRights::kingside(color))
    }

    #[inline]
    pub fn can_castle_long(&self, color: Color) -> bool {
        self.state
            .castling_rights
            .contains(CastlingRights::queenside(color))
    }

    #[inline(always)]
    pub fn en_passant_square(&self) -> Option<Square> {
        self.state.en_passant_square
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    pub fn zobrist_hash_raw(&self) -> u64 {
        self.state.zobrist_hash
    }

    pub fn is_insufficient_material(&self) -> bool {
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

        false
    }

    #[inline]
    pub fn is_threefold_repetition(&self) -> bool {
        self.repetition_count() >= 2
    }

    #[inline]
    pub fn is_twofold_repetition(&self) -> bool {
        self.repetition_count() >= 1
    }

    #[inline]
    pub fn is_fifty_move_draw(&self) -> bool {
        self.state.halfmove_clock >= crate::FIFTY_MOVE_THRESHOLD
    }

    #[inline]
    pub fn is_draw(&self) -> bool {
        self.is_fifty_move_draw()
            || self.is_threefold_repetition()
            || self.is_insufficient_material()
    }

    #[inline(always)]
    pub fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    #[inline(always)]
    pub fn halfmove_clock(&self) -> u16 {
        self.state.halfmove_clock
    }

    #[inline]
    pub fn castling_rights(&self) -> CastlingRights {
        self.state.castling_rights
    }

    /// True when `color`'s king is currently attacked.
    ///
    /// For the side to move this is the cached `checkers` set; for the opponent
    /// it has to be recomputed, since `checkers` only tracks the mover.
    #[inline(always)]
    pub fn is_in_check(&self, color: Color) -> bool {
        if color == self.side_to_move {
            !self.state.checkers.is_empty()
        } else {
            let king_sq = self.get_king_square(color);
            self.is_square_attacked(king_sq, color.opponent())
        }
    }

    /// Pieces giving check to the side to move.
    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.state.checkers
    }

    /// Own pieces that stand between `color`'s king and an enemy slider.
    #[inline(always)]
    pub fn blockers_for_king(&self, color: Color) -> BitBoard {
        self.state.blockers_for_king[color as usize]
    }

    /// Enemy pieces pinning one of `color`'s pieces to its king.
    #[inline(always)]
    pub fn pinners(&self, color: Color) -> BitBoard {
        self.state.pinners[color as usize]
    }

    /// Pieces of `color` attacking `sq`.
    #[inline]
    pub fn attackers_to_square(&self, sq: Square, color: Color) -> BitBoard {
        attacks::attackers_to_square(sq, color, self.cache.occupied, &self.pieces[color as usize])
    }

    /// Plies played since the board was created, including committed history.
    #[inline]
    pub fn ply(&self) -> usize {
        self.zobrist_history.len()
    }

    /// How many earlier positions in the current fifty-move window match the
    /// present one (same side to move).
    pub fn repetition_count(&self) -> usize {
        let history_len = self.zobrist_history.len();

        if history_len == 0 {
            return 0;
        }

        let mut count = 0;
        let look_back = (self.state.halfmove_clock as usize).min(history_len);
        let min_idx = history_len - look_back;

        let same_side_parity = history_len % 2;

        for i in min_idx..history_len {
            if i % 2 == same_side_parity && self.zobrist_history[i] == self.state.zobrist_hash {
                count += 1;
            }
        }

        count
    }

    fn are_bishops_on_same_color(&self) -> bool {
        let white_bishop_bb = &self.pieces[Color::White as usize][Piece::Bishop as usize];
        let black_bishop_bb = &self.pieces[Color::Black as usize][Piece::Bishop as usize];

        if white_bishop_bb.count() != 1 || black_bishop_bb.count() != 1 {
            return false;
        }

        let white_sq = white_bishop_bb.lsb();
        let black_sq = black_bishop_bb.lsb();

        let white_square_parity =
            (white_sq.file().to_index() + white_sq.rank().to_index()).is_multiple_of(2);
        let black_square_parity =
            (black_sq.file().to_index() + black_sq.rank().to_index()).is_multiple_of(2);

        white_square_parity == black_square_parity
    }
}
