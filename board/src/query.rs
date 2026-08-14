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

    #[inline(always)]
    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        !self.attackers_to_square(square, by_color).is_empty()
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

    #[inline]
    pub fn game_phase(&self) -> i32 {
        self.state.game_phase as i32
    }

    #[inline(always)]
    pub fn pst_scores(&self) -> (i32, i32) {
        (self.state.pst_mg, self.state.pst_eg)
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
