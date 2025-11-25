//! Transposition Table for storing search results

use aether_core::{Move, Score};

/// Type of node stored in the transposition table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Exact score (PV node)
    Exact,
    /// Lower bound (fail-high, cut node)
    LowerBound,
    /// Upper bound (fail-low, all node)
    UpperBound,
}

/// Entry in the transposition table
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    /// Zobrist hash key (used for verification)
    pub key: u64,
    /// Best move found
    pub best_move: Option<Move>,
    /// Score of the position
    pub score: Score,
    /// Depth of search
    pub depth: u8,
    /// Type of node
    pub node_type: NodeType,
    /// Age/generation of the entry
    pub age: u8,
}

impl TTEntry {
    /// Create a new TT entry
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

    /// Adjust mate scores based on ply (for correct mate distance)
    pub fn score_to_tt(score: Score, ply: usize) -> Score {
        if score > 90000 {
            score + ply as Score
        } else if score < -90000 {
            score - ply as Score
        } else {
            score
        }
    }

    /// Adjust mate scores when retrieving from TT
    pub fn score_from_tt(score: Score, ply: usize) -> Score {
        if score > 90000 {
            score - ply as Score
        } else if score < -90000 {
            score + ply as Score
        } else {
            score
        }
    }
}

/// Transposition Table
pub struct TranspositionTable {
    /// Table entries
    entries: Vec<Option<TTEntry>>,
    /// Number of entries
    size: usize,
    /// Current generation/age
    generation: u8,
    /// Number of entries used
    used: usize,
}

impl TranspositionTable {
    /// Create a new transposition table with the given size in MB
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        // Round down to power of 2 for fast indexing
        let size = num_entries.next_power_of_two() / 2;

        Self {
            entries: vec![None; size],
            size,
            generation: 0,
            used: 0,
        }
    }

    /// Get the index for a given hash key
    #[inline]
    fn index(&self, key: u64) -> usize {
        (key as usize) & (self.size - 1)
    }

    /// Probe the transposition table
    pub fn probe(&self, key: u64) -> Option<&TTEntry> {
        let idx = self.index(key);
        self.entries[idx].as_ref().filter(|e| e.key == key)
    }

    /// Store an entry in the transposition table
    pub fn store(&mut self, entry: TTEntry) {
        let idx = self.index(entry.key);

        // Replacement strategy:
        // 1. Always replace if empty
        // 2. Always replace if same position
        // 3. Replace if new entry has higher depth or is from newer generation
        let should_replace = match &self.entries[idx] {
            None => {
                self.used += 1;
                true
            }
            Some(existing) => {
                existing.key == entry.key
                    || entry.depth >= existing.depth
                    || entry.age != existing.age
            }
        };

        if should_replace {
            self.entries[idx] = Some(entry);
        }
    }

    /// Clear the transposition table
    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.used = 0;
        self.generation = 0;
    }

    /// Increment the generation (called at the start of each search)
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Get the current generation
    pub fn generation(&self) -> u8 {
        self.generation
    }

    /// Get hash table usage in permille (0-1000)
    pub fn hashfull(&self) -> u16 {
        ((self.used as u64 * 1000) / self.size as u64) as u16
    }

    /// Get the number of entries in use
    pub fn len(&self) -> usize {
        self.used
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// Resize the table to a new size in MB
    pub fn resize(&mut self, size_mb: usize) {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        let size = num_entries.next_power_of_two() / 2;

        self.entries = vec![None; size];
        self.size = size;
        self.used = 0;
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
