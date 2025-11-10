//! Cached board data for performance optimization.
//!
//! This module provides caching of frequently computed board properties
//! like combined occupancy bitboards and check status.

use aether_types::{BitBoard, Color};

/// Cache for frequently accessed board properties.
///
/// Stores precomputed bitboards and check status to avoid recomputation.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardCache {
    pub color_combined: [BitBoard; 2],
    pub occupied: BitBoard,
    pub cached_check_status: [Option<bool>; 2],
}

impl BoardCache {
    /// Creates a new empty cache.
    pub fn new() -> Self {
        Self {
            color_combined: [BitBoard::EMPTY; 2],
            occupied: BitBoard::EMPTY,
            cached_check_status: [None; 2],
        }
    }

    /// Refreshes cached occupancy bitboards from current piece positions.
    pub fn refresh(&mut self, pieces: &[[BitBoard; 6]; 2]) {
        for &color in &[Color::White, Color::Black] {
            self.color_combined[color as usize] =
                aether_types::combine_piece_bitboards(pieces[color as usize]);
        }
        self.occupied = self.color_combined[0] | self.color_combined[1];
    }

    /// Invalidates the cached check status.
    ///
    /// Should be called after moves that might change check status.
    pub fn invalidate_check_cache(&mut self) {
        self.cached_check_status = [None; 2];
    }

    /// Returns cached check status for a color if available.
    #[allow(dead_code)]
    pub fn get_cached_check_status(&self, color: Color) -> Option<bool> {
        self.cached_check_status[color as usize]
    }

    /// Caches the check status for a color.
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
