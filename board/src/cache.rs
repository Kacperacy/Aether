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
        assert!(cache.occupied.has(sq));
        assert!(cache.color_combined[Color::White as usize].has(sq));
        assert!(!cache.color_combined[Color::Black as usize].has(sq));

        cache.remove_piece(sq, Color::White);
        assert!(!cache.occupied.has(sq));
    }
}
