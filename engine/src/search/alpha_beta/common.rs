use crate::search::MAX_PLY;
use aether_core::{Move, Score};

/// Average number of legal moves in a chess position
pub const AVG_LEGAL_MOVES: usize = 40;

/// Mask for periodic node count checks
pub const NODE_CHECK_MASK: u64 = 0xFFF;

/// PV table for collecting principal variation
pub struct PvTable {
    table: [[Move; MAX_PLY]; MAX_PLY],
    length: [usize; MAX_PLY],
}

impl PvTable {
    pub fn new() -> Self {
        Self {
            table: [[Move::default(); MAX_PLY]; MAX_PLY],
            length: [0; MAX_PLY],
        }
    }

    #[inline]
    pub fn clear_ply(&mut self, ply: usize) {
        if ply < MAX_PLY {
            self.length[ply] = 0;
        }
    }

    #[inline]
    pub fn update(&mut self, ply: usize, mv: Move, limit: usize) {
        if ply >= limit {
            return;
        }

        self.table[ply][0] = mv;
        let child_len = self.length.get(ply + 1).copied().unwrap_or(0).min(MAX_PLY - ply - 2);
        for i in 0..child_len {
            self.table[ply][i + 1] = self.table[ply + 1][i];
        }
        self.length[ply] = child_len + 1;
    }

    #[inline]
    pub fn get_pv(&self) -> Vec<Move> {
        let len = self.length[0];
        self.table[0][..len].to_vec()
    }

    #[inline]
    pub fn get_best_move(&self) -> Option<Move> {
        if self.length[0] > 0 {
            Some(self.table[0][0])
        } else {
            None
        }
    }

    #[inline]
    pub fn pv_length(&self, ply: usize) -> usize {
        self.length.get(ply).copied().unwrap_or(0)
    }

    #[inline]
    pub fn get_move(&self, ply: usize, idx: usize) -> Move {
        self.table[ply][idx]
    }

    #[inline]
    pub fn set_length(&mut self, ply: usize, len: usize) {
        if ply < MAX_PLY {
            self.length[ply] = len;
        }
    }
}

impl Default for PvTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple MVV-LVA move ordering for captures
#[inline]
pub fn mvv_lva_score(mv: &Move) -> Score {
    if let Some(captured) = mv.capture {
        10 * captured.value() - mv.piece.value()
    } else if let Some(promo) = mv.promotion {
        promo.value()
    } else {
        0
    }
}

/// Order moves by simple MVV-LVA (for pure alpha-beta)
pub fn order_moves_mvv_lva(moves: &mut [Move]) {
    moves.sort_unstable_by(|a, b| {
        let a_score = mvv_lva_score(a);
        let b_score = mvv_lva_score(b);
        b_score.cmp(&a_score)
    });
}

/// Pre-allocated move lists for each ply
pub struct MoveListStack {
    lists: Vec<Vec<Move>>,
}

impl MoveListStack {
    pub fn new() -> Self {
        let lists = (0..MAX_PLY)
            .map(|_| Vec::with_capacity(AVG_LEGAL_MOVES))
            .collect();
        Self { lists }
    }

    #[inline]
    pub fn get(&mut self, ply: usize) -> &mut Vec<Move> {
        &mut self.lists[ply]
    }

    #[inline]
    pub fn take(&mut self, ply: usize) -> Vec<Move> {
        std::mem::take(&mut self.lists[ply])
    }

    #[inline]
    pub fn put_back(&mut self, ply: usize, list: Vec<Move>) {
        self.lists[ply] = list;
    }
}

impl Default for MoveListStack {
    fn default() -> Self {
        Self::new()
    }
}
