use std::io;
use std::io::{ErrorKind, Read, Seek};
use std::path::Path;

/// A move in Polyglot format
#[derive(Debug, Clone, Copy)]
pub struct PolyglotMove {
    /// Source file (0-7 = a-h)
    pub from_file: u8,
    /// Source rank (0-7 = 1-8)
    pub from_rank: u8,
    /// Destination file (0-7 = a-h)
    pub to_file: u8,
    /// Destination rank (0-7 = 1-8)
    pub to_rank: u8,
    /// Promotion piece (0=none, 1=knight, 2=bishop, 3=rook, 4=queen)
    pub promotion: u8,
}

impl PolyglotMove {
    /// Parse a move from Polyglot's 16-bit format
    fn from_u16(mv: u16) -> Self {
        Self {
            to_file: (mv & 0x07) as u8,
            to_rank: ((mv >> 3) & 0x07) as u8,
            from_file: ((mv >> 6) & 0x07) as u8,
            from_rank: ((mv >> 9) & 0x07) as u8,
            promotion: ((mv >> 12) & 0x07) as u8,
        }
    }

    /// Convert to UCI notation (e.g., "e2e4", "e7e8q")
    pub fn to_uci(&self) -> String {
        let from_file = (b'a' + self.from_file) as char;
        let from_rank = (b'1' + self.from_rank) as char;
        let to_file = (b'a' + self.to_file) as char;
        let to_rank = (b'1' + self.to_rank) as char;

        let mut s = format!("{}{}{}{}", from_file, from_rank, to_file, to_rank);

        match self.promotion {
            1 => s.push('n'),
            2 => s.push('b'),
            3 => s.push('r'),
            4 => s.push('q'),
            _ => {}
        }

        s
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PolyglotEntry {
    key: u64,
    mv: u16,
    weight: u16,
    learn: u32,
}

impl PolyglotEntry {
    /// Get the move from this entry
    pub fn get_move(&self) -> PolyglotMove {
        PolyglotMove::from_u16(self.mv)
    }

    /// Get the weight (probability) of this move
    pub fn weight(&self) -> u16 {
        self.weight
    }
}

pub struct OpeningBook {
    file: std::fs::File,
    entry_count: usize,
}

impl PolyglotEntry {
    const SIZE: usize = 16;

    fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE);

        Self {
            key: u64::from_be_bytes(bytes[0..8].try_into().unwrap()),
            mv: u16::from_be_bytes(bytes[8..10].try_into().unwrap()),
            weight: u16::from_be_bytes(bytes[10..12].try_into().unwrap()),
            learn: u32::from_be_bytes(bytes[12..16].try_into().unwrap()),
        }
    }
}

impl OpeningBook {
    /// Opens a polyglot opening book from the specified file path
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let file_size = file.metadata()?.len() as usize;

        if file_size % PolyglotEntry::SIZE != 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid polyglot book file size",
            ));
        }

        let entry_count = file_size / PolyglotEntry::SIZE;

        Ok(Self { file, entry_count })
    }

    /// Returns the number of entries in the book
    pub fn len(&self) -> usize {
        self.entry_count
    }

    /// Returns true if the book is empty
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }

    /// Reads a polyglot entry at the specified index
    fn read_entry(&mut self, index: usize) -> io::Result<PolyglotEntry> {
        let offset = (index * PolyglotEntry::SIZE) as u64;
        self.file.seek(io::SeekFrom::Start(offset))?;

        let mut buffer = [0u8; PolyglotEntry::SIZE];
        self.file.read_exact(&mut buffer)?;

        Ok(PolyglotEntry::from_bytes(&buffer))
    }

    /// Finds the first occurrence of an entry with the specified key using binary search
    fn find_first(&mut self, key: u64) -> io::Result<Option<usize>> {
        let mut left = 0;
        let mut right = self.entry_count;
        let mut result: Option<usize> = None;

        while left < right {
            let mid = (left + right) / 2;
            let entry = self.read_entry(mid)?;

            if entry.key < key {
                left = mid + 1;
            } else if entry.key > key {
                right = mid;
            } else {
                result = Some(mid);
                right = mid;
            }
        }

        Ok(result)
    }

    /// Probes the opening book for all entries matching the specified key
    pub fn probe(&mut self, key: u64) -> io::Result<Vec<PolyglotEntry>> {
        let first_index = match self.find_first(key)? {
            Some(idx) => idx,
            None => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        let mut index = first_index;

        while index < self.entry_count {
            let entry = self.read_entry(index)?;
            if entry.key != key {
                break;
            }
            entries.push(entry);
            index += 1;
        }

        Ok(entries)
    }

    /// Select a move from the book based on weights
    /// Uses weighted random selection - higher weight = higher probability
    pub fn select_move(&mut self, key: u64) -> io::Result<Option<PolyglotMove>> {
        let entries = self.probe(key)?;

        if entries.is_empty() {
            return Ok(None);
        }

        // Calculate total weight
        let total_weight: u32 = entries.iter().map(|e| e.weight as u32).sum();

        if total_weight == 0 {
            // All weights are 0, just pick the first move
            return Ok(Some(entries[0].get_move()));
        }

        // Simple weighted selection using a deterministic approach
        // In a real engine, you'd use a random number generator
        // For now, we'll just pick the highest-weighted move
        let best_entry = entries.iter().max_by_key(|e| e.weight).unwrap();

        Ok(Some(best_entry.get_move()))
    }

    /// Select a random move from the book based on weights
    /// Uses weighted random selection with the provided random value (0.0 to 1.0)
    pub fn select_move_weighted(
        &mut self,
        key: u64,
        random: f64,
    ) -> io::Result<Option<PolyglotMove>> {
        let entries = self.probe(key)?;

        if entries.is_empty() {
            return Ok(None);
        }

        let total_weight: u32 = entries.iter().map(|e| e.weight as u32).sum();

        if total_weight == 0 {
            return Ok(Some(entries[0].get_move()));
        }

        let threshold = (random * total_weight as f64) as u32;
        let mut cumulative = 0u32;

        for entry in &entries {
            cumulative += entry.weight as u32;
            if cumulative > threshold {
                return Ok(Some(entry.get_move()));
            }
        }

        // Fallback to last entry
        Ok(Some(entries.last().unwrap().get_move()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polyglot_move_parsing() {
        // e2e4 in polyglot format
        // to_file=4, to_rank=3, from_file=4, from_rank=1
        // = 4 | (3 << 3) | (4 << 6) | (1 << 9) = 4 | 24 | 256 | 512 = 796
        let mv = PolyglotMove::from_u16(796);
        assert_eq!(mv.from_file, 4); // e
        assert_eq!(mv.from_rank, 1); // 2
        assert_eq!(mv.to_file, 4); // e
        assert_eq!(mv.to_rank, 3); // 4
        assert_eq!(mv.promotion, 0);
        assert_eq!(mv.to_uci(), "e2e4");
    }

    #[test]
    fn test_promotion_move() {
        // e7e8q - promotion to queen
        // to_file=4, to_rank=7, from_file=4, from_rank=6, promo=4
        let mv_raw = 4 | (7 << 3) | (4 << 6) | (6 << 9) | (4 << 12);
        let mv = PolyglotMove::from_u16(mv_raw);
        assert_eq!(mv.to_uci(), "e7e8q");
    }
}
