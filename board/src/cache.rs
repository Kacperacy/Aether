use aether_core::{ALL_COLORS, BitBoard, Color, Square};

/// Cached aggregate bitboards for fast access during move generation and evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct BoardCache {
    /// Combined bitboard for each color (all pieces of that color)
    pub color_combined: [BitBoard; 2],
    /// All occupied squares
    pub occupied: BitBoard,
}

impl BoardCache {
    /// Creates a new, empty BoardCache
    #[inline]
    pub const fn new() -> Self {
        Self {
            color_combined: [BitBoard::EMPTY; 2],
            occupied: BitBoard::EMPTY,
        }
    }

    /// Fully refreshes the cache based on the provided piece bitboards
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

    /// Incrementally updates the cache when a piece is added
    #[inline]
    pub fn add_piece(&mut self, square: Square, color: Color) {
        let bb = BitBoard::from_square(square);
        self.color_combined[color as usize] |= bb;
        self.occupied |= bb;
    }

    /// Incrementally updates the cache when a piece is removed
    #[inline]
    pub fn remove_piece(&mut self, square: Square, color: Color) {
        let bb = BitBoard::from_square(square);
        self.color_combined[color as usize] &= !bb;
        self.occupied &= !bb;
    }

    /// Incrementally updates the cache when a piece moves
    #[inline]
    pub fn move_piece(&mut self, from: Square, to: Square, color: Color) {
        let from_bb = BitBoard::from_square(from);
        let to_bb = BitBoard::from_square(to);
        let combined = from_bb | to_bb;

        self.color_combined[color as usize] ^= combined;
        self.occupied ^= combined;
    }

    /// Returns true if the square is occupied
    #[inline]
    pub fn is_occupied(&self, square: Square) -> bool {
        self.occupied.has(square)
    }

    /// Returns true if the square is occupied by a piece of the given color
    #[inline]
    pub fn is_occupied_by(&self, square: Square, color: Color) -> bool {
        self.color_combined[color as usize].has(square)
    }

    /// Returns the color of the piece on the square, if any
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
