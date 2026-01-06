use aether_core::{ALL_COLORS, BitBoard, Color, Square};

#[derive(Debug, Clone, PartialEq)]
pub struct BoardCache {
    pub color_combined: [BitBoard; 2],
    pub occupied: BitBoard,
}

impl BoardCache {
    #[inline]
    pub const fn new() -> Self {
        Self {
            color_combined: [BitBoard::EMPTY; 2],
            occupied: BitBoard::EMPTY,
        }
    }

    pub fn refresh(&mut self, pieces: &[[BitBoard; 6]; 2]) {
        for color in ALL_COLORS {
            self.color_combined[color as usize] = pieces[color as usize][0]
                | pieces[color as usize][1]
                | pieces[color as usize][2]
                | pieces[color as usize][3]
                | pieces[color as usize][4]
                | pieces[color as usize][5];
        }
        self.occupied = self.color_combined[0] | self.color_combined[1];
    }

    #[inline]
    pub fn add_piece(&mut self, square: Square, color: Color) {
        let bb = square.bitboard();
        self.color_combined[color as usize] |= bb;
        self.occupied |= bb;
    }

    #[inline]
    pub fn remove_piece(&mut self, square: Square, color: Color) {
        let bb = square.bitboard();
        self.color_combined[color as usize] &= !bb;
        self.occupied &= !bb;
    }

    #[inline]
    pub fn move_piece(&mut self, from: Square, to: Square, color: Color) {
        let from_bb = from.bitboard();
        let to_bb = to.bitboard();
        let combined = from_bb | to_bb;

        self.color_combined[color as usize] ^= combined;
        self.occupied ^= combined;
    }

    #[inline]
    pub fn is_occupied(&self, square: Square) -> bool {
        self.occupied.has(square)
    }

    #[inline]
    pub fn is_occupied_by(&self, square: Square, color: Color) -> bool {
        self.color_combined[color as usize].has(square)
    }

    #[inline]
    pub fn color_at(&self, square: Square) -> Option<Color> {
        if self.color_combined[Color::White as usize].has(square) {
            Some(Color::White)
        } else if self.color_combined[Color::Black as usize].has(square) {
            Some(Color::Black)
        } else {
            None
        }
    }
}

impl Default for BoardCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = BoardCache::new();
        assert!(cache.occupied.is_empty());
        assert!(cache.color_combined[0].is_empty());
        assert!(cache.color_combined[1].is_empty());
    }

    #[test]
    fn test_cache_add_remove() {
        let mut cache = BoardCache::new();
        let sq = Square::E4;

        cache.add_piece(sq, Color::White);
        assert!(cache.is_occupied(sq));
        assert!(cache.is_occupied_by(sq, Color::White));
        assert!(!cache.is_occupied_by(sq, Color::Black));

        cache.remove_piece(sq, Color::White);
        assert!(!cache.is_occupied(sq));
    }

    #[test]
    fn test_cache_move_piece() {
        let mut cache = BoardCache::new();
        let from = Square::E2;
        let to = Square::E4;

        cache.add_piece(from, Color::White);
        cache.move_piece(from, to, Color::White);

        assert!(!cache.is_occupied(from));
        assert!(cache.is_occupied(to));
        assert!(cache.is_occupied_by(to, Color::White));
    }

    #[test]
    fn test_cache_color_at() {
        let mut cache = BoardCache::new();

        cache.add_piece(Square::E2, Color::White);
        cache.add_piece(Square::E7, Color::Black);

        assert_eq!(cache.color_at(Square::E2), Some(Color::White));
        assert_eq!(cache.color_at(Square::E7), Some(Color::Black));
        assert_eq!(cache.color_at(Square::E4), None);
    }
}
