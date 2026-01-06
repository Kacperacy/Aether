use crate::{Color, File, Piece, Square};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

#[derive(Debug, Clone)]
pub struct ZobristKeys {
    /// [square][piece][color] - 64 squares, 6 pieces, 2 colors
    pub pieces: [[[u64; 2]; 6]; 64],
    pub side_to_move: u64,
    pub castling: [[u64; 2]; 2],
    pub en_passant: [u64; 8],
}

impl ZobristKeys {
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

    #[inline(always)]
    pub fn piece_key(&self, square: Square, piece: Piece, color: Color) -> u64 {
        self.pieces[square.to_index() as usize][piece as usize][color as usize]
    }

    #[inline(always)]
    pub fn castling_key(&self, color: Color, kingside: bool) -> u64 {
        let side = if kingside { 0 } else { 1 };
        self.castling[color as usize][side]
    }

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

use std::sync::OnceLock;

static ZOBRIST_KEYS: OnceLock<ZobristKeys> = OnceLock::new();

pub fn zobrist_keys() -> &'static ZobristKeys {
    ZOBRIST_KEYS.get_or_init(ZobristKeys::new)
}
