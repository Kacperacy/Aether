use aether_types::{ALL_COLORS, BitBoard, Color, combine_piece_bitboards};

#[derive(Debug, Clone, PartialEq)]
pub struct BoardCache {
    pub color_combined: [BitBoard; 2],
    pub occupied: BitBoard,
    pub cached_check_status: [Option<bool>; 2],
}

impl BoardCache {
    /// Creates a new, empty BoardCache
    pub fn new() -> Self {
        Self {
            color_combined: [BitBoard::EMPTY; 2],
            occupied: BitBoard::EMPTY,
            cached_check_status: [None; 2],
        }
    }

    /// Fully refreshes the cache based on the provided piece bitboards
    pub fn refresh(&mut self, pieces: &[[BitBoard; 6]; 2]) {
        for color in ALL_COLORS {
            self.color_combined[color as usize] = combine_piece_bitboards(pieces[color as usize]);
        }
        self.occupied = self.color_combined[0] | self.color_combined[1];
    }

    /// Invalidates the cached check status for both colors
    pub fn invalidate_check_cache(&mut self) {
        self.cached_check_status = [None; 2];
    }

    /// Retrieves the cached check status for the specified color, if available
    #[allow(dead_code)]
    pub fn get_cached_check_status(&self, color: Color) -> Option<bool> {
        self.cached_check_status[color as usize]
    }

    /// Sets the cached check status for the specified color
    #[allow(dead_code)]
    pub fn set_cached_check_status(&mut self, color: Color, in_check: bool) {
        self.cached_check_status[color as usize] = Some(in_check);
    }
}

impl Default for BoardCache {
    /// Creates a default BoardCache (empty)
    fn default() -> Self {
        Self::new()
    }
}
