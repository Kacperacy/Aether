use std::io;
use std::io::{ErrorKind, Read, Seek};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct PolyglotEntry {
    key: u64,
    mv: u16,
    weight: u16,
    learn: u32,
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

    // pub fn select_move(&mut self, key: u64) -> io::Result<Option<PolyglotMove>> {}
}
