use aether_types::Move;
use eval::Score;

/// Type of transposition table entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Exact score (PV node)
    Exact,
    /// Lower bound (beta cutoff, fail-high)
    LowerBound,
    /// Upper bound (fail-low)
    UpperBound,
}

/// Entry in the transposition table
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    /// Zobrist hash of the position
    pub hash: u64,

    /// Best move found in this position
    pub best_move: Option<Move>,

    /// Score of the position
    pub score: Score,

    /// Depth at which this position was searched
    pub depth: u8,

    /// Type of score (exact, lower bound, upper bound)
    pub entry_type: EntryType,

    /// Age of the entry (for replacement strategy)
    pub age: u8,
}

impl TTEntry {
    /// Create a new transposition table entry
    pub fn new(
        hash: u64,
        best_move: Option<Move>,
        score: Score,
        depth: u8,
        entry_type: EntryType,
        age: u8,
    ) -> Self {
        Self {
            hash,
            best_move,
            score,
            depth,
            entry_type,
            age,
        }
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            hash: 0,
            best_move: None,
            score: 0,
            depth: 0,
            entry_type: EntryType::Exact,
            age: 0,
        }
    }
}

/// Transposition Table
///
/// A hash table that stores previously searched positions to avoid
/// redundant work. This is one of the most important optimizations
/// in chess engines, often providing a 5-10x speedup.
pub struct TranspositionTable {
    /// The table entries
    entries: Vec<TTEntry>,

    /// Size of the table (number of entries)
    size: usize,

    /// Current age (incremented each search)
    age: u8,

    /// Number of hits
    hits: u64,

    /// Number of misses
    misses: u64,
}

impl TranspositionTable {
    /// Create a new transposition table with the given size in MB
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        // Round down to nearest power of 2 for efficient modulo
        let size = num_entries.next_power_of_two() / 2;

        Self {
            entries: vec![TTEntry::default(); size],
            size,
            age: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Create a default transposition table (16 MB)
    pub fn default_size() -> Self {
        Self::new(16)
    }

    /// Get the index for a hash value
    #[inline(always)]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) % self.size
    }

    /// Probe the transposition table for a position
    ///
    /// Returns Some(entry) if the position is found, None otherwise
    pub fn probe(&mut self, hash: u64) -> Option<&TTEntry> {
        let index = self.index(hash);
        let entry = &self.entries[index];

        if entry.hash == hash {
            self.hits += 1;
            Some(entry)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Store an entry in the transposition table
    ///
    /// Uses a replacement strategy that prefers:
    /// 1. Empty slots
    /// 2. Older entries
    /// 3. Entries searched to lower depth
    pub fn store(
        &mut self,
        hash: u64,
        best_move: Option<Move>,
        score: Score,
        depth: u8,
        entry_type: EntryType,
    ) {
        let index = self.index(hash);
        let existing = &self.entries[index];

        // Always replace if:
        // 1. Slot is empty (hash == 0)
        // 2. Same position (hash match)
        // 3. Deeper search
        // 4. Older age
        let should_replace = existing.hash == 0
            || existing.hash == hash
            || depth >= existing.depth
            || existing.age < self.age;

        if should_replace {
            self.entries[index] = TTEntry::new(
                hash,
                best_move,
                score,
                depth,
                entry_type,
                self.age,
            );
        }
    }

    /// Clear the transposition table
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = TTEntry::default();
        }
        self.hits = 0;
        self.misses = 0;
    }

    /// Increment the age (call at the start of each search)
    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    /// Get the hash table usage (per-mille, 0-1000)
    pub fn hash_full(&self) -> u16 {
        let mut used = 0;
        let sample_size = 1000.min(self.size);

        for i in 0..sample_size {
            if self.entries[i].hash != 0 {
                used += 1;
            }
        }

        ((used * 1000) / sample_size) as u16
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get statistics
    pub fn stats(&self) -> TTStats {
        TTStats {
            size: self.size,
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.hit_rate(),
            hash_full: self.hash_full(),
        }
    }
}

/// Transposition table statistics
#[derive(Debug, Clone)]
pub struct TTStats {
    pub size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub hash_full: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_creation() {
        let tt = TranspositionTable::new(1);
        assert!(tt.size > 0);
        assert_eq!(tt.age, 0);
    }

    #[test]
    fn test_tt_store_and_probe() {
        let mut tt = TranspositionTable::new(1);
        let hash = 12345u64;

        // Store an entry
        tt.store(hash, None, 100, 5, EntryType::Exact);

        // Probe should find it
        let entry = tt.probe(hash);
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.hash, hash);
        assert_eq!(entry.score, 100);
        assert_eq!(entry.depth, 5);
    }

    #[test]
    fn test_tt_replacement() {
        let mut tt = TranspositionTable::new(1);
        let hash = 12345u64;

        // Store shallow search
        tt.store(hash, None, 50, 3, EntryType::Exact);

        // Store deeper search - should replace
        tt.store(hash, None, 100, 5, EntryType::Exact);

        let entry = tt.probe(hash).unwrap();
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 100);
    }

    #[test]
    fn test_tt_age() {
        let mut tt = TranspositionTable::new(1);
        assert_eq!(tt.age, 0);

        tt.new_search();
        assert_eq!(tt.age, 1);

        tt.new_search();
        assert_eq!(tt.age, 2);
    }

    #[test]
    fn test_tt_hash_full() {
        let mut tt = TranspositionTable::new(1);
        assert_eq!(tt.hash_full(), 0);

        // Fill some entries
        for i in 0..100 {
            tt.store(i, None, 0, 1, EntryType::Exact);
        }

        assert!(tt.hash_full() > 0);
    }
}
