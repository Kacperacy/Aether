use aether_types::{ALL_COLORS, BitBoard, Color};

#[derive(Debug, Clone, PartialEq)]
pub struct BoardCache {
    pub color_combined: [BitBoard; 2],
    pub occupied: BitBoard,
    pub cached_check_status: [Option<bool>; 2],
}

impl BoardCache {
    pub fn new() -> Self {
        Self {
            color_combined: [BitBoard::EMPTY; 2],
            occupied: BitBoard::EMPTY,
            cached_check_status: [None; 2],
        }
    }

    pub fn refresh(&mut self, pieces: &[[BitBoard; 6]; 2]) {
        for color in ALL_COLORS {
            self.color_combined[color as usize] =
                aether_types::combine_piece_bitboards(pieces[color as usize]);
        }
        self.occupied = self.color_combined[0] | self.color_combined[1];
    }

    pub fn invalidate_check_cache(&mut self) {
        self.cached_check_status = [None; 2];
    }

    #[allow(dead_code)]
    pub fn get_cached_check_status(&self, color: Color) -> Option<bool> {
        self.cached_check_status[color as usize]
    }

    #[allow(dead_code)]
    pub fn set_check_status(&mut self, color: Color, in_check: bool) {
        self.cached_check_status[color as usize] = Some(in_check);
    }
}

impl Default for BoardCache {
    fn default() -> Self {
        Self::new()
    }
}
