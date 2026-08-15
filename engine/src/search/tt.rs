use crate::eval::{MATE_THRESHOLD, Score};
use aether_core::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeType {
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

impl NodeType {
    #[inline]
    fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self::Exact,
            1 => Self::LowerBound,
            _ => Self::UpperBound,
        }
    }
}

/// A transposition-table slot, packed to exactly 16 bytes.
///
/// The size is the point. Slots are indexed by a mask, so the table holds a
/// power-of-two number of them; with the previous 24-byte layout a power-of-two
/// count could never fill a power-of-two byte budget, and every hash size lost
/// exactly 25% of the memory it asked for. At 16 bytes the count divides the
/// budget exactly, so a 16MB table now holds 1048576 entries instead of 524288.
///
/// `best_move` is stored as raw bits with `Move::NULL` meaning "none", which
/// costs nothing: `Move::NULL` encodes a1a1 and can never be legal.
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub key: u64,
    score: Score,
    best_move: u16,
    pub depth: u8,
    /// `age << 2 | node_type`. Age wraps at 64 rather than 256; replacement only
    /// ever compares ages for equality, so the narrower counter is equivalent.
    packed: u8,
}

const _: () = assert!(
    std::mem::size_of::<TTEntry>() == 16,
    "TTEntry must stay 16 bytes or the table silently wastes hash again"
);

impl TTEntry {
    /// An unoccupied slot. A real position hashing to key 0 is a 1-in-2^64
    /// event, and costs one wasted slot rather than any incorrect result.
    pub const EMPTY: Self = Self {
        key: 0,
        score: 0,
        best_move: 0,
        depth: 0,
        packed: 0,
    };

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
            score,
            best_move: match best_move {
                Some(mv) => mv.0,
                None => Move::NULL.0,
            },
            depth,
            packed: (age << 2) | node_type as u8,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.key == 0
    }

    #[inline]
    pub fn score(&self) -> Score {
        self.score
    }

    #[inline]
    pub fn best_move(&self) -> Option<Move> {
        let mv = Move(self.best_move);
        if mv == Move::NULL { None } else { Some(mv) }
    }

    #[inline]
    pub fn node_type(&self) -> NodeType {
        NodeType::from_bits(self.packed)
    }

    #[inline]
    pub fn age(&self) -> u8 {
        self.packed >> 2
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
    entries: Vec<TTEntry>,
    size: usize,
    generation: u8,
}

/// Largest power-of-two slot count fitting in `size_mb`, and never zero —
/// `index` masks with `size - 1`, which would underflow on an empty table.
fn entry_count(size_mb: usize) -> usize {
    let fits = (size_mb * 1024 * 1024) / std::mem::size_of::<TTEntry>();

    if fits.is_power_of_two() {
        fits.max(1)
    } else {
        (fits.next_power_of_two() / 2).max(1)
    }
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let size = entry_count(size_mb);

        Self {
            entries: vec![TTEntry::EMPTY; size],
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
        let entry = &self.entries[idx];

        // An empty slot has key 0, so a real key never matches one.
        if entry.key == key { Some(entry) } else { None }
    }

    #[inline]
    pub fn store(&mut self, entry: TTEntry) {
        let idx = self.index(entry.key);
        let existing = &self.entries[idx];

        let should_replace = existing.is_empty()
            || existing.key == entry.key
            || entry.depth >= existing.depth
            || (entry.age() != existing.age() && entry.depth + 3 >= existing.depth);

        if should_replace {
            self.entries[idx] = entry;
        }
    }

    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::EMPTY);
        self.generation = 0;
    }

    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn generation(&self) -> u8 {
        self.generation
    }

    /// Occupancy in permille, as UCI `hashfull` expects.
    ///
    /// Samples across the whole table rather than the first 1000 slots. Both are
    /// unbiased while every slot is equally likely to be hit, but a stride keeps
    /// the figure meaningful if indexing ever stops being a plain mask.
    pub fn hashfull(&self) -> u16 {
        const SAMPLE_SIZE: usize = 1000;
        let sample_count = SAMPLE_SIZE.min(self.size);
        let stride = self.size / sample_count;

        let filled = (0..sample_count)
            .filter(|i| !self.entries[i * stride].is_empty())
            .count();

        ((filled * 1000) / sample_count) as u16
    }

    /// Number of slots in the table. Always a power of two, never zero.
    pub fn capacity(&self) -> usize {
        self.size
    }

    pub fn resize(&mut self, size_mb: usize) {
        let size = entry_count(size_mb);

        self.entries = vec![TTEntry::EMPTY; size];
        self.size = size;
        self.generation = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::Square;

    #[test]
    fn test_tt_basic() {
        let mut tt = TranspositionTable::new(1);

        let entry = TTEntry::new(12345, None, 100, 5, NodeType::Exact, 0);

        tt.store(entry);

        let probe = tt.probe(12345);
        assert!(probe.is_some());

        let probe = probe.unwrap();
        assert_eq!(probe.score(), 100);
        assert_eq!(probe.depth, 5);
    }

    #[test]
    fn test_tt_miss() {
        let tt = TranspositionTable::new(1);
        assert!(tt.probe(99999).is_none());
    }

    /// A power-of-two hash request must be used in full. The 24-byte layout
    /// could not manage this: a power-of-two slot count times 24 bytes never
    /// fills a power-of-two byte budget, so every size lost exactly 25%.
    #[test]
    fn test_power_of_two_hash_sizes_are_fully_used() {
        for mb in [1usize, 2, 4, 16, 64, 256, 1024] {
            let tt = TranspositionTable::new(mb);
            let used = tt.capacity() * std::mem::size_of::<TTEntry>();

            assert_eq!(
                used,
                mb * 1024 * 1024,
                "{mb}MB request used {used} bytes, wasting {}%",
                100 - (used * 100) / (mb * 1024 * 1024)
            );
        }
    }

    /// `index` masks with `size - 1`, so a zero-length table would underflow.
    #[test]
    fn test_table_is_never_empty() {
        for mb in [0usize, 1] {
            let tt = TranspositionTable::new(mb);
            assert!(tt.capacity() >= 1, "{mb}MB produced an empty table");
            assert!(tt.capacity().is_power_of_two());
            assert!(tt.probe(12345).is_none());
        }
    }

    #[test]
    fn test_resize_matches_a_fresh_table_and_drops_contents() {
        let mut tt = TranspositionTable::new(1);
        tt.store(TTEntry::new(12345, None, 100, 5, NodeType::Exact, 0));

        tt.resize(4);

        assert_eq!(tt.capacity(), TranspositionTable::new(4).capacity());
        assert!(tt.probe(12345).is_none(), "resize must not keep stale data");
    }

    /// Every field must survive the 16-byte packing intact.
    #[test]
    fn test_packed_fields_round_trip() {
        let mv = Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH);

        for (node_type, age) in [
            (NodeType::Exact, 0u8),
            (NodeType::LowerBound, 1),
            (NodeType::UpperBound, 63),
        ] {
            let entry = TTEntry::new(0xDEAD_BEEF, Some(mv), -12345, 42, node_type, age);

            assert_eq!(entry.key, 0xDEAD_BEEF);
            assert_eq!(entry.score(), -12345);
            assert_eq!(entry.best_move(), Some(mv));
            assert_eq!(entry.depth, 42);
            assert_eq!(entry.node_type(), node_type);
            assert_eq!(entry.age(), age);
            assert!(!entry.is_empty());
        }
    }

    /// `None` is stored as `Move::NULL`, which is safe precisely because
    /// `Move::NULL` encodes a1a1 and can never be a legal move.
    #[test]
    fn test_absent_best_move_round_trips_as_none() {
        let entry = TTEntry::new(1, None, 0, 1, NodeType::Exact, 0);
        assert_eq!(entry.best_move(), None);
        assert!(TTEntry::EMPTY.is_empty());
        assert_eq!(TTEntry::EMPTY.best_move(), None);
    }

    #[test]
    fn test_hashfull_reports_empty_and_full() {
        let mut tt = TranspositionTable::new(1);
        assert_eq!(tt.hashfull(), 0);

        for key in 1..=tt.capacity() as u64 {
            tt.store(TTEntry::new(key, None, 0, 1, NodeType::Exact, 0));
        }
        assert!(tt.hashfull() > 900, "got {}", tt.hashfull());

        tt.clear();
        assert_eq!(tt.hashfull(), 0);
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
