use crate::{Color, File, Piece, Square};

#[derive(Debug, Clone)]
pub struct ZobristKeys {
    /// [square][piece][color] - 64 squares, 6 pieces, 2 colors
    pub pieces: [[[u64; 2]; 6]; 64],
    pub side_to_move: u64,
    pub castling: [[u64; 2]; 2],
    pub en_passant: [u64; 8],
}

/// SplitMix64. Chosen because it is a `const fn`, which is what lets the whole
/// key set be built by the compiler.
///
/// Returns the advanced state alongside the output, rather than taking `&mut`,
/// so it composes cleanly inside a `const` initialiser.
const fn splitmix64(state: u64) -> (u64, u64) {
    let state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);

    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);

    (state, z ^ (z >> 31))
}

impl ZobristKeys {
    const fn generate() -> Self {
        let mut keys = ZobristKeys {
            pieces: [[[0; 2]; 6]; 64],
            side_to_move: 0,
            castling: [[0; 2]; 2],
            en_passant: [0; 8],
        };

        let mut state = 0x517c_c1b7_2722_0a95;

        let (s, v) = splitmix64(state);
        state = s;
        keys.side_to_move = v;

        let mut square = 0;
        while square < 64 {
            let mut piece = 0;
            while piece < 6 {
                let mut color = 0;
                while color < 2 {
                    let (s, v) = splitmix64(state);
                    state = s;
                    keys.pieces[square][piece][color] = v;
                    color += 1;
                }
                piece += 1;
            }
            square += 1;
        }

        let mut color = 0;
        while color < 2 {
            let mut side = 0;
            while side < 2 {
                let (s, v) = splitmix64(state);
                state = s;
                keys.castling[color][side] = v;
                side += 1;
            }
            color += 1;
        }

        let mut file = 0;
        while file < 8 {
            let (s, v) = splitmix64(state);
            state = s;
            keys.en_passant[file] = v;
            file += 1;
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

/// The keys, built at compile time.
///
/// This used to be a `OnceLock` seeded from ChaCha8, which put an atomic acquire
/// on every access — and `make_move` accesses it four to ten times, once per
/// hash toggle. On aarch64 that is a real `ldar` each time, not a free load.
/// A `const` initialiser removes both the atomic and the startup work.
static ZOBRIST_KEYS: ZobristKeys = ZobristKeys::generate();

#[inline(always)]
pub fn zobrist_keys() -> &'static ZobristKeys {
    &ZOBRIST_KEYS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Zobrist hashing is only sound if the keys are distinct: any repeat means
    /// two different positions collide by construction rather than by chance.
    #[test]
    fn test_all_keys_are_distinct() {
        let keys = zobrist_keys();
        let mut seen = HashSet::new();

        seen.insert(keys.side_to_move);

        for square in 0..64 {
            for piece in 0..6 {
                for color in 0..2 {
                    let key = keys.pieces[square][piece][color];
                    assert!(seen.insert(key), "duplicate piece key at {square}/{piece}");
                    assert_ne!(key, 0, "zero key at {square}/{piece}");
                }
            }
        }

        for color in 0..2 {
            for side in 0..2 {
                assert!(
                    seen.insert(keys.castling[color][side]),
                    "duplicate castling"
                );
            }
        }

        for file in 0..8 {
            assert!(seen.insert(keys.en_passant[file]), "duplicate en passant");
        }

        assert_eq!(seen.len(), 1 + 64 * 6 * 2 + 4 + 8);
    }

    /// A crude bias check: across the whole key set each bit position should be
    /// set roughly half the time. Catches a generator that is broken enough to
    /// produce structured keys.
    #[test]
    fn test_key_bits_are_balanced() {
        let keys = zobrist_keys();
        let mut counts = [0usize; 64];
        let mut total = 0usize;

        for square in 0..64 {
            for piece in 0..6 {
                for color in 0..2 {
                    let key = keys.pieces[square][piece][color];
                    for (bit, count) in counts.iter_mut().enumerate() {
                        *count += ((key >> bit) & 1) as usize;
                    }
                    total += 1;
                }
            }
        }

        for (bit, &count) in counts.iter().enumerate() {
            let ratio = count as f64 / total as f64;
            assert!(
                (0.40..0.60).contains(&ratio),
                "bit {bit} set in {:.1}% of keys",
                ratio * 100.0
            );
        }
    }
}
