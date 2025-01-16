use crate::board::{Board, Color};
use once_cell::sync::Lazy;
use rand::{rng, Rng};

pub struct Zobrist {
    pub pieces: [[u64; 64]; 12],
    pub castling_rights: [u64; 16],
    pub en_passant: [u64; 8],
    pub side: u64,
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = rng();
        let mut pieces = [[0; 64]; 12];
        let mut castling_rights = [0; 16];
        let mut en_passant = [0; 8];
        let side = rng.random();

        for i in 0..12 {
            for j in 0..64 {
                pieces[i][j] = rng.random();
            }
        }

        for i in 0..16 {
            castling_rights[i] = rng.random();
        }

        for i in 0..8 {
            en_passant[i] = rng.random();
        }

        Self {
            pieces,
            castling_rights,
            en_passant,
            side,
        }
    }

    pub fn hash(&self, board: &Board) -> u64 {
        let mut hash = 0;
        let occupancy =
            board.occupancy[Color::White as usize] | board.occupancy[Color::Black as usize];

        for i in 0..64 {
            if occupancy.is_set(i) {
                let piece = board.piece_at(i).unwrap();
                hash ^= self.pieces[piece.piece as usize * (1 + piece.color as usize)][i];
            }
        }

        if board.turn == Color::Black {
            hash ^= self.side;
        }

        for i in 0..4 {
            if board.castling_rights & (1 << i) != 0 {
                hash ^= self.castling_rights[i];
            }
        }

        if let Some(en_passant) = board.en_passant_square {
            hash ^= self.en_passant[en_passant % 8];
        }

        hash
    }
}

pub static ZOBRIST: Lazy<Zobrist> = Lazy::new(Zobrist::new);
