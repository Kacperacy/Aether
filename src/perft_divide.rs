use aether_core::{BISHOP_MAGICS, BISHOP_MOVES, BitBoard, Square, bishop_attacks};

pub fn main() {
    let f1 = Square::F1;
    let sq_idx = f1.to_index() as usize;

    // Position after 1.e3 a6
    // Occupied squares on bishop f1 diagonals (from previous analysis):
    // e2: NOT occupied (pawn moved)
    // g2: occupied by pawn
    // d3: not occupied
    // c4: not occupied
    // b5: not occupied
    // a6: occupied by black pawn
    // h3: not occupied

    // Build occupied for the position
    let mut occupied = BitBoard::EMPTY;
    // Only g2 is relevant in mask (e2 is in mask and empty, g2 is in mask and occupied)
    occupied |= BitBoard::from_square(Square::G2);
    // a6 and h3 are NOT in mask so they don't matter

    println!("Testing bishop attacks for f1");
    println!("Occupied (just g2): {:064b}", occupied.value());

    let magic = &BISHOP_MAGICS[sq_idx];
    println!("\nMagic entry for f1:");
    println!("  mask: {:064b}", magic.mask);
    println!("  magic: 0x{:016X}", magic.magic);
    println!("  index_bits: {}", magic.index_bits);

    // Manual index calculation
    let relevant = occupied.value() & magic.mask;
    println!("\nRelevant occupancy:");
    println!("  occupied & mask = {:064b}", relevant);

    let hash = relevant.wrapping_mul(magic.magic);
    println!("  hash = {:064b}", hash);

    let index = hash >> (64 - magic.index_bits);
    println!("  index = {}", index);

    // What's in table at that index?
    let moves = &BISHOP_MOVES[sq_idx];
    println!("\nTable size: {}", moves.len());
    println!("Attacks at index {}: {:064b}", index, moves[index as usize]);

    let attacks = BitBoard(moves[index as usize]);
    println!("\nAttack squares:");
    for sq in attacks {
        println!("  {}", sq);
    }

    // Now test with library function
    println!("\n\nUsing bishop_attacks function:");
    let lib_attacks = bishop_attacks(f1, occupied);
    println!("Attacks: {:064b}", lib_attacks.value());
    for sq in lib_attacks {
        println!("  {}", sq);
    }

    // Expected: e2, d3, c4, b5, a6 (blocked at g2, so not h3)
    // should have 5 squares: e2, d3, c4, b5, a6
}
