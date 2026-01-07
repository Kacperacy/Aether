use crate::{ALL_SQUARES, BitBoard, File, Rank, Square};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::Write;
use std::path::Path;
use std::{fs, io};

#[derive(Debug, Clone)]
struct MagicEntry {
    mask: BitBoard,
    magic: u64,
    index_bits: u8,
    attacks: Vec<BitBoard>,
}

fn rook_mask(sq: Square) -> BitBoard {
    let mut mask = BitBoard::EMPTY;
    let rank = sq.rank() as i8;
    let file = sq.file() as i8;

    for r in (rank + 1)..7 {
        mask |= Square::new(File::from_index(file), Rank::from_index(r)).bitboard();
    }

    for r in (1..rank).rev() {
        mask |= Square::new(File::from_index(file), Rank::from_index(r)).bitboard();
    }

    for f in (file + 1)..7 {
        mask |= Square::new(File::from_index(f), Rank::from_index(rank)).bitboard();
    }

    for f in (1..file).rev() {
        mask |= Square::new(File::from_index(f), Rank::from_index(rank)).bitboard();
    }

    mask
}

fn bishop_mask(sq: Square) -> BitBoard {
    let mut mask = BitBoard::EMPTY;
    let rank = sq.rank() as i8;
    let file = sq.file() as i8;

    for i in 1..7 {
        if rank + i < 7 && file + i < 7 {
            mask |= Square::new(File::from_index(file + i), Rank::from_index(rank + i)).bitboard();
        }

        if rank + i < 7 && file - i > 0 {
            mask |= Square::new(File::from_index(file - i), Rank::from_index(rank + i)).bitboard();
        }

        if rank - i > 0 && file + i < 7 {
            mask |= Square::new(File::from_index(file + i), Rank::from_index(rank - i)).bitboard();
        }

        if rank - i > 0 && file - i > 0 {
            mask |= Square::new(File::from_index(file - i), Rank::from_index(rank - i)).bitboard();
        }
    }

    mask
}

fn generate_blockers(mask: BitBoard) -> Vec<BitBoard> {
    let bits: Vec<Square> = mask.iter().collect();
    let count = 1 << bits.len();
    let mut blockers = Vec::with_capacity(count);

    for i in 0..count {
        let mut bb = BitBoard::EMPTY;
        for (j, &sq) in bits.iter().enumerate() {
            if (i & (1 << j)) != 0 {
                bb |= sq.bitboard();
            }
        }
        blockers.push(bb);
    }

    blockers
}

fn rook_attacks(sq: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard::EMPTY;
    let rank = sq.rank() as i8;
    let file = sq.file() as i8;

    for r in (rank + 1)..=7 {
        let target = Square::new(File::from_index(file), Rank::from_index(r));
        attacks |= target.bitboard();
        if blockers.has(target) {
            break;
        }
    }

    for r in (0..rank).rev() {
        let target = Square::new(File::from_index(file), Rank::from_index(r));
        attacks |= target.bitboard();
        if blockers.has(target) {
            break;
        }
    }

    for f in (file + 1)..=7 {
        let target = Square::new(File::from_index(f), Rank::from_index(rank));
        attacks |= target.bitboard();
        if blockers.has(target) {
            break;
        }
    }

    for f in (0..file).rev() {
        let target = Square::new(File::from_index(f), Rank::from_index(rank));
        attacks |= target.bitboard();
        if blockers.has(target) {
            break;
        }
    }

    attacks
}

fn bishop_attacks(sq: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard::EMPTY;
    let rank = sq.rank() as i8;
    let file = sq.file() as i8;

    // Up-Right diagonal
    for i in 1..=7 {
        if rank + i <= 7 && file + i <= 7 {
            let target = Square::new(File::from_index(file + i), Rank::from_index(rank + i));
            attacks |= target.bitboard();
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    // Up-Left diagonal
    for i in 1..=7 {
        if rank + i <= 7 && file - i >= 0 {
            let target = Square::new(File::from_index(file - i), Rank::from_index(rank + i));
            attacks |= target.bitboard();
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    // Down-Right diagonal
    for i in 1..=7 {
        if rank - i >= 0 && file + i <= 7 {
            let target = Square::new(File::from_index(file + i), Rank::from_index(rank - i));
            attacks |= target.bitboard();
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    // Down-Left diagonal
    for i in 1..=7 {
        if rank - i >= 0 && file - i >= 0 {
            let target = Square::new(File::from_index(file - i), Rank::from_index(rank - i));
            attacks |= target.bitboard();
            if blockers.has(target) {
                break;
            }
        } else {
            break;
        }
    }

    attacks
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

    let bits = mask.count() as u8;
    let table_size = 1 << bits;

    let seed = 42 + sq.to_index() as u64 + if is_rook { 0 } else { 64 };
    let mut rng = StdRng::seed_from_u64(seed);

    'search: loop {
        let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

        if (magic.wrapping_mul(mask.value()) >> 56).count_ones() < 6 {
            continue;
        }

        let mut table = vec![None; table_size];

        for (i, &blocker) in blockers.iter().enumerate() {
            let index = magic_index(mask, magic, bits, blocker);

            match table[index] {
                None => table[index] = Some(attacks[i]),
                Some(existing) if existing == attacks[i] => {
                    continue;
                }
                Some(_) => {
                    continue 'search;
                }
            }
        }

        let attack_table = table
            .into_iter()
            .map(|opt| opt.unwrap_or(BitBoard::EMPTY))
            .collect();

        return MagicEntry {
            mask,
            magic,
            index_bits: bits,
            attacks: attack_table,
        };
    }
}

#[inline]
fn magic_index(mask: BitBoard, magic: u64, bits: u8, blockers: BitBoard) -> usize {
    let relevant = blockers.value() & mask.value();
    let hash = relevant.wrapping_mul(magic);
    (hash >> (64 - bits)) as usize
}

fn generate_all_magics() -> (Vec<MagicEntry>, Vec<MagicEntry>) {
    println!("Generating all magics...");

    let mut rook_magics = Vec::with_capacity(64);
    let mut bishop_magics = Vec::with_capacity(64);

    for (i, &sq) in ALL_SQUARES.iter().enumerate() {
        print!("\rProgress: {}/64", i + 1);
        std::io::stdout().flush().unwrap();

        rook_magics.push(find_magic(sq, true));
        bishop_magics.push(find_magic(sq, false));
    }

    println!("\nMagic generation complete.");
    (rook_magics, bishop_magics)
}

fn generate_code(rook_magics: &[MagicEntry], bishop_magics: &[MagicEntry]) -> String {
    let mut code = String::with_capacity(1024 * 1024);

    code.push_str("//! Auto-generated magic numbers for move generation\n");
    code.push_str("//! DO NOT EDIT MANUALLY - regenerate with codegen feature\n\n");
    code.push_str("//! Generated by: core/src/attacks/codegen.rs\n\n");

    code.push_str("use super::magic::MagicEntry;\n\n");

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const ROOK_MAGICS: &[MagicEntry; 64] = &[\n");
    for entry in rook_magics {
        code.push_str(&format!(
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:016X}, index_bits: {} }},\n",
            entry.mask.value(),
            entry.magic,
            entry.index_bits
        ));
    }
    code.push_str("];\n\n");

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const BISHOP_MAGICS: &[MagicEntry; 64] = &[\n");
    for entry in bishop_magics {
        code.push_str(&format!(
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:016X}, index_bits: {} }},\n",
            entry.mask.value(),
            entry.magic,
            entry.index_bits
        ));
    }
    code.push_str("];\n\n");

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const ROOK_MOVES: &[&[u64]; 64] = &[\n");
    for entry in rook_magics {
        code.push_str("    &[");
        for (i, attack) in entry.attacks.iter().enumerate() {
            if i % 8 == 0 {
                code.push_str("\n        ");
            }
            code.push_str(&format!("0x{:016X}, ", attack.value()));
        }
        code.push_str("],\n");
    }
    code.push_str("];\n\n");

    code.push_str("#[rustfmt::skip]\n");
    code.push_str("pub const BISHOP_MOVES: &[&[u64]; 64] = &[\n");
    for entry in bishop_magics {
        code.push_str("    &[");
        for (i, attack) in entry.attacks.iter().enumerate() {
            if i % 8 == 0 {
                code.push_str("\n        ");
            }
            code.push_str(&format!("0x{:016X}, ", attack.value()));
        }
        code.push_str("],\n");
    }
    code.push_str("];\n");

    code
}

pub fn generate_magic_constants<P: AsRef<Path>>(output_path: P) -> io::Result<()> {
    let path = output_path.as_ref();

    let (rook_magics, bishop_magics) = generate_all_magics();

    let code = generate_code(&rook_magics, &bishop_magics);

    let mut file = fs::File::create(path)?;
    file.write_all(code.as_bytes())?;

    println!("Magic constants written to {:?}", path.display());

    let rook_total: usize = rook_magics.iter().map(|e| e.attacks.len()).sum();
    let bishop_total: usize = bishop_magics.iter().map(|e| e.attacks.len()).sum();

    println!("Statistics:");
    println!("  Rook table entries: {}", rook_total);
    println!("  Bishop table entries: {}", bishop_total);
    println!("  Total table entries: {}", rook_total + bishop_total);
    println!(
        "  Approximate size: {} KB",
        (rook_total + bishop_total) * 8 / 1024
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook_mask() {
        let mask = rook_mask(Square::E4);
        // Should not include edges
        assert!(!mask.has(Square::E1));
        assert!(!mask.has(Square::E8));
        assert!(!mask.has(Square::A4));
        assert!(!mask.has(Square::H4));

        // Should include inner squares
        assert!(mask.has(Square::E2));
        assert!(mask.has(Square::E7));
        assert!(mask.has(Square::B4));
        assert!(mask.has(Square::G4));
    }

    #[test]
    fn test_generate_blockers() {
        let mask = Square::E4.bitboard() | Square::E5.bitboard();
        let blockers = generate_blockers(mask);

        // 2 bits → 4 combinations
        assert_eq!(blockers.len(), 4);

        // Should include all combinations
        assert!(blockers.contains(&BitBoard::EMPTY));
        assert!(blockers.contains(&Square::E4.bitboard()));
        assert!(blockers.contains(&Square::E5.bitboard()));
        assert!(blockers.contains(&(Square::E4.bitboard() | Square::E5.bitboard())));
    }

    #[test]
    fn test_magic_entry_quality() {
        let entry = find_magic(Square::E4, true);

        // Magic should be non-zero
        assert_ne!(entry.magic, 0);

        // Table size should match 2^index_bits
        assert_eq!(entry.attacks.len(), 1 << entry.index_bits);

        // Mask should have reasonable number of bits
        assert!(entry.mask.count() >= 8);
        assert!(entry.mask.count() <= 12);
    }
}
