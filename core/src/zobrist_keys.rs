use crate::{Color, File, Piece, Square};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

/// Zobrist keys for hashing chess positions
/// Uses deterministic random generation with fixed seed for consistency
#[derive(Debug, Clone)]
pub struct ZobristKeys {
    /// [square][piece][color] - 64 squares, 6 pieces, 2 colors
    pub pieces: [[[u64; 2]; 6]; 64],
    /// Side to move hash
    pub side_to_move: u64,
    /// Castling rights [color][side] - 2 colors, 2 sides (kingside/queenside)
    pub castling: [[u64; 2]; 2],
    /// En passant file hash (8 files)
    pub en_passant: [u64; 8],
}

impl ZobristKeys {
    /// Initialize zobrist keys with deterministic random values
    pub fn new() -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(0x517cc1b727220a95);

        let mut keys = ZobristKeys {
            pieces: [[[0; 2]; 6]; 64],
            side_to_move: rng.random(),
            castling: [[0; 2]; 2],
            en_passant: [0; 8],
        };

        // Generate piece-square keys
        for square in 0..64 {
            for piece in 0..6 {
                for color in 0..2 {
                    keys.pieces[square][piece][color] = rng.random();
                }
            }
        }

        // Generate castling keys
        for color in 0..2 {
            for side in 0..2 {
                keys.castling[color][side] = rng.random();
            }
        }

        // Generate en passant keys
        for file in 0..8 {
            keys.en_passant[file] = rng.random();
        }

        keys
    }

    /// Get piece key for specific square, piece, and color
    #[inline(always)]
    pub fn piece_key(&self, square: Square, piece: Piece, color: Color) -> u64 {
        self.pieces[square.to_index() as usize][piece as usize][color as usize]
    }

    /// Get castling key for specific color and side
    #[inline(always)]
    pub fn castling_key(&self, color: Color, kingside: bool) -> u64 {
        let side = if kingside { 0 } else { 1 };
        self.castling[color as usize][side]
    }

    /// Get en passant key for specific file
    #[inline(always)]
    pub fn en_passant_key(&self, file: File) -> u64 {
        self.en_passant[file.to_index() as usize]
    }
}

impl Default for ZobristKeys {
    fn default() -> Self {
        Self::new()
    }
}

/// Global zobrist keys instance - lazy static for thread safety
use std::sync::OnceLock;

static ZOBRIST_KEYS: OnceLock<ZobristKeys> = OnceLock::new();

/// Get global zobrist keys instance
pub fn zobrist_keys() -> &'static ZobristKeys {
    ZOBRIST_KEYS.get_or_init(ZobristKeys::new)
}
