use aether_core::{MATE_THRESHOLD, Move, Score};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub key: u64,
    pub best_move: Option<Move>,
    pub score: Score,
    pub depth: u8,
    pub node_type: NodeType,
    pub age: u8,
}

impl TTEntry {
    pub fn new(
        key: u64,
        best_move: Option<Move>,
        score: Score,
        depth: u8,
        node_type: NodeType,
        age: u8,
    ) -> Self {
        Self {
            key,
            best_move,
            score,
            depth,
            node_type,
            age,
        }
    }

    #[inline]
    pub fn score_to_tt(score: Score, ply: usize) -> Score {
        if score > MATE_THRESHOLD {
            score + ply as Score
        } else if score < -MATE_THRESHOLD {
            score - ply as Score
        } else {
            score
        }
    }

    #[inline]
    pub fn score_from_tt(score: Score, ply: usize) -> Score {
        if score > MATE_THRESHOLD {
            score - ply as Score
        } else if score < -MATE_THRESHOLD {
            score + ply as Score
        } else {
            score
        }
    }
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    size: usize,
    generation: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        let size = if num_entries.is_power_of_two() {
            num_entries
        } else {
            num_entries.next_power_of_two() / 2
        };

        Self {
            entries: vec![None; size],
            size,
            generation: 0,
        }
    }

    #[inline]
    fn index(&self, key: u64) -> usize {
        (key as usize) & (self.size - 1)
    }

    #[inline]
    pub fn prefetch(&self, key: u64) {
        let idx = self.index(key);
        let ptr = self.entries.as_ptr().wrapping_add(idx) as *const i8;

        #[cfg(target_arch = "x86_64")]
        unsafe {
            std::arch::x86_64::_mm_prefetch(ptr, std::arch::x86_64::_MM_HINT_T0);
        }

        #[cfg(target_arch = "x86")]
        unsafe {
            std::arch::x86::_mm_prefetch(ptr, std::arch::x86::_MM_HINT_T0);
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
        let _ = ptr;
    }

    #[inline]
    pub fn probe(&self, key: u64) -> Option<&TTEntry> {
        let idx = self.index(key);
        self.entries[idx].as_ref().filter(|e| e.key == key)
    }

    #[inline]
    pub fn store(&mut self, entry: TTEntry) {
        let idx = self.index(entry.key);

        let should_replace = match &self.entries[idx] {
            None => true,
            Some(existing) => {
                existing.key == entry.key
                    || entry.depth >= existing.depth
                    || (entry.age != existing.age && entry.depth + 3 >= existing.depth)
            }
        };

        if should_replace {
            self.entries[idx] = Some(entry);
        }
    }

    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.generation = 0;
    }

    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn generation(&self) -> u8 {
        self.generation
    }

    pub fn hashfull(&self) -> u16 {
        const SAMPLE_SIZE: usize = 1000;
        let sample_count = SAMPLE_SIZE.min(self.size);

        let filled = self.entries[..sample_count]
            .iter()
            .filter(|e| e.is_some())
            .count();

        ((filled * 1000) / sample_count) as u16
    }

    pub fn resize(&mut self, size_mb: usize) {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        let size = if num_entries.is_power_of_two() {
            num_entries
        } else {
            num_entries.next_power_of_two() / 2
        };

        self.entries = vec![None; size];
        self.size = size;
        self.generation = 0;
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(16) // 16 MB default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_basic() {
        let mut tt = TranspositionTable::new(1);

        let entry = TTEntry::new(12345, None, 100, 5, NodeType::Exact, 0);

        tt.store(entry);

        let probe = tt.probe(12345);
        assert!(probe.is_some());

        let probe = probe.unwrap();
        assert_eq!(probe.score, 100);
        assert_eq!(probe.depth, 5);
    }

    #[test]
    fn test_tt_miss() {
        let tt = TranspositionTable::new(1);
        assert!(tt.probe(99999).is_none());
    }

    #[test]
    fn test_mate_score_adjustment() {
        // Mate in 3 at ply 0
        let mate_score = 99997;
        let tt_score = TTEntry::score_to_tt(mate_score, 0);
        let retrieved = TTEntry::score_from_tt(tt_score, 0);
        assert_eq!(retrieved, mate_score);

        // Same mate score retrieved at different ply
        let tt_score = TTEntry::score_to_tt(mate_score, 2);
        let retrieved = TTEntry::score_from_tt(tt_score, 2);
        assert_eq!(retrieved, mate_score);
    }
}
