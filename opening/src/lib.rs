//! Opening book crate
//!
//! Responsibilities:
//! - Provide opening book (library of good opening moves)
//! - Support querying book moves for a given position
//! - Simple text-based format for easy editing

use std::collections::HashMap;

/// A book move with weight (higher weight = more likely to play)
#[derive(Debug, Clone)]
pub struct BookMove {
    /// Move in UCI notation (e.g., "e2e4")
    pub mv: String,
    
    /// Weight (higher = better/more popular)
    pub weight: u32,
    
    /// Optional comment/annotation
    pub comment: Option<String>,
}

/// Opening book
pub struct OpeningBook {
    /// Map from Zobrist hash to list of book moves
    positions: HashMap<u64, Vec<BookMove>>,
}

impl OpeningBook {
    /// Create an empty opening book
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }
    
    /// Create opening book with default openings
    pub fn default_book() -> Self {
        let mut book = Self::new();
        book.add_default_openings();
        book
    }
    
    /// Add a move to the book
    pub fn add_move(&mut self, hash: u64, mv: String, weight: u32, comment: Option<String>) {
        let book_move = BookMove { mv, weight, comment };
        
        self.positions
            .entry(hash)
            .or_default()
            .push(book_move);
    }
    
    /// Get book moves for a position
    pub fn get_moves(&self, hash: u64) -> Option<&[BookMove]> {
        self.positions.get(&hash).map(|v| v.as_slice())
    }
    
    /// Pick a random move weighted by book weights
    pub fn pick_move(&self, hash: u64) -> Option<String> {
        let moves = self.get_moves(hash)?;
        
        if moves.is_empty() {
            return None;
        }
        
        // Calculate total weight
        let total_weight: u32 = moves.iter().map(|m| m.weight).sum();
        
        if total_weight == 0 {
            // All weights are 0, pick uniformly
            return Some(moves[0].mv.clone());
        }
        
        // Pick weighted random
        // For simplicity, just pick the highest weight move
        // In a full implementation, use rand crate for true randomness
        
        
        moves
            .iter()
            .max_by_key(|m| m.weight)
            .map(|m| m.mv.clone())
    }
    
    /// Get number of positions in book
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    
    /// Check if book is empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
    
    /// Add common opening theory
    fn add_default_openings(&mut self) {
        // Starting position hash - we'll use 0 as placeholder
        // In real usage, you'd compute actual Zobrist hashes
        let start_hash = 0u64;
        
        // Common opening moves from starting position
        self.add_move(start_hash, "e2e4".to_string(), 100, Some("King's Pawn".to_string()));
        self.add_move(start_hash, "d2d4".to_string(), 90, Some("Queen's Pawn".to_string()));
        self.add_move(start_hash, "g1f3".to_string(), 70, Some("Réti Opening".to_string()));
        self.add_move(start_hash, "c2c4".to_string(), 60, Some("English Opening".to_string()));
        self.add_move(start_hash, "e2e3".to_string(), 30, Some("Van't Kruijs Opening".to_string()));
        self.add_move(start_hash, "b2b3".to_string(), 20, Some("Nimzowitsch-Larsen Attack".to_string()));
        
        // After 1.e4 e5
        // Hash would be different, using placeholder
        let e4e5_hash = 1u64;
        self.add_move(e4e5_hash, "g1f3".to_string(), 100, Some("King's Knight".to_string()));
        self.add_move(e4e5_hash, "f1c4".to_string(), 70, Some("Bishop's Opening".to_string()));
        self.add_move(e4e5_hash, "b1c3".to_string(), 50, Some("Vienna Game".to_string()));
        
        // After 1.d4 d5
        let d4d5_hash = 2u64;
        self.add_move(d4d5_hash, "c2c4".to_string(), 100, Some("Queen's Gambit".to_string()));
        self.add_move(d4d5_hash, "g1f3".to_string(), 80, Some("Queen's Pawn Game".to_string()));
        self.add_move(d4d5_hash, "c1f4".to_string(), 40, Some("London System".to_string()));
    }
}

impl Default for OpeningBook {
    fn default() -> Self {
        Self::default_book()
    }
}

/// Load opening book from text format
///
/// Format:
/// ```text
/// # Comment
/// hash move weight [comment]
/// 0 e2e4 100 King's Pawn
/// 0 d2d4 90 Queen's Pawn
/// ```
pub fn load_book_from_text(text: &str) -> OpeningBook {
    let mut book = OpeningBook::new();
    
    for line in text.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        
        // Parse hash, move, weight
        let hash: u64 = parts[0].parse().unwrap_or(0);
        let mv = parts[1].to_string();
        let weight: u32 = parts[2].parse().unwrap_or(1);
        let comment = parts.get(3).map(|s| s.to_string());
        
        book.add_move(hash, mv, weight, comment);
    }
    
    book
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_book() {
        let book = OpeningBook::new();
        assert_eq!(book.len(), 0);
        assert!(book.is_empty());
    }
    
    #[test]
    fn test_add_move() {
        let mut book = OpeningBook::new();
        book.add_move(0, "e2e4".to_string(), 100, Some("Test".to_string()));
        
        assert_eq!(book.len(), 1);
        let moves = book.get_moves(0).unwrap();
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].mv, "e2e4");
        assert_eq!(moves[0].weight, 100);
    }
    
    #[test]
    fn test_pick_move() {
        let mut book = OpeningBook::new();
        book.add_move(0, "e2e4".to_string(), 100, None);
        book.add_move(0, "d2d4".to_string(), 50, None);
        
        let mv = book.pick_move(0);
        assert!(mv.is_some());
        // Should pick highest weight
        assert_eq!(mv.unwrap(), "e2e4");
    }
    
    #[test]
    fn test_default_book() {
        let book = OpeningBook::default_book();
        assert!(!book.is_empty());
        
        // Should have starting position moves
        let moves = book.get_moves(0);
        assert!(moves.is_some());
        assert!(!moves.unwrap().is_empty());
    }
    
    #[test]
    fn test_load_from_text() {
        let text = r#"
# Test book
0 e2e4 100 King's Pawn
0 d2d4 90 Queen's Pawn
1 g1f3 80
        "#;
        
        let book = load_book_from_text(text);
        assert_eq!(book.len(), 2); // 2 unique hashes
        
        let moves0 = book.get_moves(0).unwrap();
        assert_eq!(moves0.len(), 2);
        
        let moves1 = book.get_moves(1).unwrap();
        assert_eq!(moves1.len(), 1);
    }
}
