use aether_types::{BitBoard, Color};

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

    pub fn update_occupancy(&mut self, pieces: &[[BitBoard; 6]; 2]) {
        self.color_combined[Color::White as usize] = pieces[Color::White as usize]
            .iter()
            .fold(BitBoard::EMPTY, |acc, &bb| acc | bb);

        self.color_combined[Color::Black as usize] = pieces[Color::Black as usize]
            .iter()
            .fold(BitBoard::EMPTY, |acc, &bb| acc | bb);

        self.occupied =
            self.color_combined[Color::White as usize] | self.color_combined[Color::Black as usize];
    }

    pub fn invalidate_check_cache(&mut self) {
        self.cached_check_status = [None; 2];
    }

    pub fn get_cached_check_status(&self, color: Color) -> Option<bool> {
        self.cached_check_status[color as usize]
    }

    pub fn set_check_status(&mut self, color: Color, in_check: bool) {
        self.cached_check_status[color as usize] = Some(in_check);
    }
}

impl Default for BoardCache {
    fn default() -> Self {
        Self::new()
    }
}
