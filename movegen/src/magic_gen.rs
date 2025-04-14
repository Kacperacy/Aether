use aether_types::{ALL_SQUARES, BitBoard, File, Rank, Square};
use rand::prelude::*;
use std::fs::File as FsFile;
use std::io::Write;
use std::path::Path;

struct MagicEntry {
    mask: BitBoard,
    magic: u64,
    index_bits: u8,
    moves: Vec<BitBoard>,
}

fn rook_mask(sq: Square) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let r = sq.rank() as i8;
    let f = sq.file() as i8;

    for i in r + 1..7 {
        result |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(i)));
    }
    for i in (1..r).rev() {
        result |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(i)));
    }
    for i in f + 1..7 {
        result |= BitBoard::from_square(Square::new(File::from_index(i), Rank::new(r)));
    }
    for i in (1..f).rev() {
        result |= BitBoard::from_square(Square::new(File::from_index(i), Rank::new(r)));
    }

    result
}

fn bishop_mask(sq: Square) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let r = sq.rank() as i8;
    let f = sq.file() as i8;

    for i in 1..7 {
        if r + i < 7 && f + i < 7 {
            result |= BitBoard::from_square(Square::new(File::from_index(f + i), Rank::new(r + i)));
        }
        if r + i < 7 && f - i > 0 {
            result |= BitBoard::from_square(Square::new(File::from_index(f - i), Rank::new(r + i)));
        }
        if r - i > 0 && f + i < 7 {
            result |= BitBoard::from_square(Square::new(File::from_index(f + i), Rank::new(r - i)));
        }
        if r - i > 0 && f - i > 0 {
            result |= BitBoard::from_square(Square::new(File::from_index(f - i), Rank::new(r - i)));
        }
    }

    result
}

fn generate_blockers(mask: BitBoard) -> Vec<BitBoard> {
    let bits: Vec<Square> = mask.into_iter().collect();
    let count = 1 << bits.len();
    let mut result = Vec::with_capacity(count);

    for i in 0..count {
        let mut blocker = BitBoard::EMPTY;
        for j in 0..bits.len() {
            if (i & (1 << j)) != 0 {
                blocker |= BitBoard::from_square(bits[j]);
            }
        }
        result.push(blocker);
    }

    result
}

fn rook_attacks(sq: Square, blockers: BitBoard) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let r = sq.rank() as i8;
    let f = sq.file() as i8;

    // Rays in four directions, stopping at blockers
    for i in r + 1..=7 {
        let target = Square::new(File::from_index(f), Rank::new(i));
        result |= BitBoard::from_square(target);
        if blockers.has(target) {
            break;
        }
    }

    for i in (0..r).rev() {
        let target = Square::new(File::from_index(f), Rank::new(i));
        result |= BitBoard::from_square(target);
        if blockers.has(target) {
            break;
        }
    }

    for i in f + 1..=7 {
        let target = Square::new(File::from_index(i), Rank::new(r));
        result |= BitBoard::from_square(target);
        if blockers.has(target) {
            break;
        }
    }

    for i in (0..f).rev() {
        let target = Square::new(File::from_index(i), Rank::new(r));
        result |= BitBoard::from_square(target);
        if blockers.has(target) {
            break;
        }
    }

    result
}

fn bishop_attacks(sq: Square, blockers: BitBoard) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let r = sq.rank() as i8;
    let f = sq.file() as i8;

    for i in 1..=7 {
        if r + i <= 7 && f + i <= 7 {
            let target = Square::new(File::from_index(f + i), Rank::new(r + i));
            result |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    for i in 1..=7 {
        if r + i <= 7 && f - i >= 0 {
            let target = Square::new(File::from_index(f - i), Rank::new(r + i));
            result |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    for i in 1..=7 {
        if r - i >= 0 && f + i <= 7 {
            let target = Square::new(File::from_index(f + i), Rank::new(r - i));
            result |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    for i in 1..=7 {
        if r - i >= 0 && f - i >= 0 {
            let target = Square::new(File::from_index(f - i), Rank::new(r - i));
            result |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    result
}

fn find_magic(sq: Square, is_rook: bool) -> MagicEntry {
    let mask = if is_rook {
        rook_mask(sq)
    } else {
        bishop_mask(sq)
    };
    let blockers = generate_blockers(mask);
    let attacks: Vec<BitBoard> = blockers
        .iter()
        .map(|&b| {
            if is_rook {
                rook_attacks(sq, b)
            } else {
                bishop_attacks(sq, b)
            }
        })
        .collect();

    let bits = mask.len();
    let size = 1 << bits;

    let mut rng = StdRng::seed_from_u64(42 + sq as u64 + if is_rook { 64 } else { 0 });

    'outer: loop {
        let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

        if (magic.wrapping_mul(mask.0) >> 56).count_ones() < 6 {
            continue;
        }

        let mut used = vec![BitBoard::EMPTY; size];
        let mut used_attacks = vec![None; size];

        for i in 0..blockers.len() {
            let blocker = blockers[i];
            let attack = attacks[i];

            let index = magic_index(mask, magic, bits as u8, blocker);

            if used_attacks[index].is_none() {
                used[index] = blocker;
                used_attacks[index] = Some(attack);
            } else if used_attacks[index] != Some(attack) {
                continue 'outer;
            }
        }

        let mut moves = vec![BitBoard::EMPTY; size];
        for i in 0..size {
            moves[i] = used_attacks[i].unwrap_or(BitBoard::EMPTY);
        }

        return MagicEntry {
            mask,
            magic,
            index_bits: bits as u8,
            moves,
        };
    }
}

fn magic_index(mask: BitBoard, magic: u64, bits: u8, blockers: BitBoard) -> usize {
    let relevant = blockers & mask;
    let hash = relevant.0.wrapping_mul(magic);
    (hash >> (64 - bits)) as usize
}

fn generate_magic_tables() -> (Vec<MagicEntry>, Vec<MagicEntry>) {
    let mut rook_magics = Vec::with_capacity(Square::NUM);
    let mut bishop_magics = Vec::with_capacity(Square::NUM);

    for sq in ALL_SQUARES.iter() {
        rook_magics.push(find_magic(*sq, true));
        bishop_magics.push(find_magic(*sq, false));
    }

    (rook_magics, bishop_magics)
}

fn generate_magic_code(rook_magics: &[MagicEntry], bishop_magics: &[MagicEntry]) -> String {
    let mut code = String::new();

    code.push_str(
        "pub struct MagicEntry {\n    pub mask: bb,\n    pub magic: u64,\n    pub index_bits: u8,\n}\n\n",
    );

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const ROOK_MAGICS: &[MagicEntry; Square::NUM] = &[\n");
    for entry in rook_magics {
        code.push_str(&format!(
            "    MagicEntry {{ mask: bb({}), magic: {}, index_bits: {} }},\n",
            entry.mask.0, entry.magic, entry.index_bits
        ));
    }
    code.push_str("];\n\n");

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const BISHOP_MAGICS: &[MagicEntry; Square::NUM] = &[\n");
    for entry in bishop_magics {
        code.push_str(&format!(
            "    MagicEntry {{ mask: bb({}), magic: {}, index_bits: {} }},\n",
            entry.mask.0, entry.magic, entry.index_bits
        ));
    }
    code.push_str("];\n\n");

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const ROOK_MOVES: &[&[bb]; Square::NUM] = &[\n");
    for entry in rook_magics {
        code.push_str("    &[\n");
        code.push_str("    ");
        for mov in &entry.moves {
            code.push_str(&format!("bb({}), ", mov.0));
        }
        code.push_str("    ],\n");
    }
    code.push_str("];\n\n");

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const BISHOP_MOVES: &[&[bb]; Square::NUM] = &[\n");
    for entry in bishop_magics {
        code.push_str("    &[\n");
        code.push_str("    ");
        for mov in &entry.moves {
            code.push_str(&format!("bb({}), ", mov.0));
        }
        code.push_str("    ],\n");
    }
    code.push_str("];\n");

    code
}

pub fn generate_magic_constants(output_path: &str) -> std::io::Result<()> {
    println!("Generating magic constants...");
    let (rook_magics, bishop_magics) = generate_magic_tables();

    let code = generate_magic_code(&rook_magics, &bishop_magics);

    let path = Path::new(output_path);
    let mut file = FsFile::create(path)?;

    writeln!(file, "// Auto-generated magic bitboard constants")?;
    writeln!(file, "// Do not edit manually")?;
    writeln!(file, "")?;
    writeln!(file, "use aether_types::{{BitBoard as bb, Square}};")?;
    writeln!(file, "")?;

    file.write_all(code.as_bytes())?;

    println!("Magic constants written to {}", output_path);
    Ok(())
}
