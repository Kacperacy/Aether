//! Search tuning parameters.
//!
//! These were scattered as private `const`s through `alpha_beta.rs`, which made
//! the set of things worth tuning invisible — and several of the real knobs were
//! never named at all, just written inline at the point of use. Collecting them
//! here is the prerequisite for driving them from an external tuner: a search
//! parameter that has no name cannot be searched over.
//!
//! Values are unchanged from where they were defined, so this move is
//! behaviour-preserving by construction and the bench node count proves it.

use crate::eval::{Score, material};

/// A parameter an external tuner is allowed to move, with the range it may move
/// it within.
pub struct Tunable {
    pub name: &'static str,
    pub default: i32,
    pub min: i32,
    pub max: i32,
}

/// The tunable set, in the same order as the `idx` constants below.
///
/// Ranges are deliberately generous: they exist to stop a tuner writing a value
/// that would crash the search (a zero divisor, a depth past the table), not to
/// encode an opinion about where the optimum is.
pub const TUNABLES: &[Tunable] = &[
    Tunable {
        name: "DeltaPruningMargin",
        default: 200,
        min: 0,
        max: 1000,
    },
    Tunable {
        name: "NullMoveReduction",
        default: 3,
        min: 1,
        max: 6,
    },
    Tunable {
        name: "NullMoveMinDepth",
        default: 3,
        min: 1,
        max: 10,
    },
    Tunable {
        name: "LmrFullDepthMoves",
        default: 4,
        min: 1,
        max: 16,
    },
    Tunable {
        name: "LmrMinDepth",
        default: 3,
        min: 1,
        max: 10,
    },
    Tunable {
        name: "LmrDivisor",
        default: 6,
        min: 1,
        max: 32,
    },
    Tunable {
        name: "AspirationDepth",
        default: 5,
        min: 1,
        max: 20,
    },
    Tunable {
        name: "AspirationWindow",
        default: 25,
        min: 1,
        max: 200,
    },
    Tunable {
        name: "AspirationMaxDelta",
        default: 400,
        min: 50,
        max: 2000,
    },
    Tunable {
        name: "FutilityMaxDepth",
        default: 3,
        min: 0,
        max: 3,
    },
    Tunable {
        name: "RfpMargin",
        default: 120,
        min: 0,
        max: 1000,
    },
    Tunable {
        name: "RfpMaxDepth",
        default: 3,
        min: 0,
        max: 10,
    },
];

pub mod idx {
    pub const DELTA_PRUNING_MARGIN: usize = 0;
    pub const NULL_MOVE_REDUCTION: usize = 1;
    pub const NULL_MOVE_MIN_DEPTH: usize = 2;
    pub const LMR_FULL_DEPTH_MOVES: usize = 3;
    pub const LMR_MIN_DEPTH: usize = 4;
    pub const LMR_DIVISOR: usize = 5;
    pub const ASPIRATION_DEPTH: usize = 6;
    pub const ASPIRATION_WINDOW: usize = 7;
    pub const ASPIRATION_MAX_DELTA: usize = 8;
    pub const FUTILITY_MAX_DEPTH: usize = 9;
    pub const RFP_MARGIN: usize = 10;
    pub const RFP_MAX_DEPTH: usize = 11;
}

/// Sentinel for "never overridden". A real parameter never takes this value, so
/// it distinguishes an unset slot from a deliberate zero.
#[cfg(feature = "tune")]
const UNSET: i32 = i32::MIN;

#[cfg(feature = "tune")]
static OVERRIDES: [std::sync::atomic::AtomicI32; TUNABLES.len()] =
    [const { std::sync::atomic::AtomicI32::new(UNSET) }; TUNABLES.len()];

/// Current value of parameter `i`.
///
/// Without the `tune` feature this is the compile-time default and every call
/// folds away, so the shipping build pays nothing for the machinery.
#[inline(always)]
pub fn value(i: usize) -> i32 {
    #[cfg(feature = "tune")]
    {
        let raw = OVERRIDES[i].load(std::sync::atomic::Ordering::Relaxed);
        if raw != UNSET {
            return raw;
        }
    }
    TUNABLES[i].default
}

/// Override a parameter by name, clamped to its declared range.
///
/// Returns false when no parameter has that name. Only available under `tune`;
/// the default build has nothing to set.
#[cfg(feature = "tune")]
pub fn set_by_name(name: &str, v: i32) -> bool {
    for (i, t) in TUNABLES.iter().enumerate() {
        if t.name.eq_ignore_ascii_case(name) {
            let clamped = v.clamp(t.min, t.max);
            OVERRIDES[i].store(clamped, std::sync::atomic::Ordering::Relaxed);
            return true;
        }
    }
    false
}

/// How often the search checks the clock and the stop flag, as a node mask.
/// Checking every node would put a clock read in the innermost loop.
pub const NODE_CHECK_MASK: u64 = 0xFFF;

// ===== Quiescence delta pruning =====

/// Slack added before writing off a capture as unable to raise alpha.
#[inline(always)]
pub fn delta_pruning_margin() -> Score {
    value(idx::DELTA_PRUNING_MARGIN)
}

/// Largest swing a single move can plausibly produce: capture a queen and
/// promote to one. Used to bail out of a hopeless quiescence node wholesale.
pub const DELTA_MAX_GAIN: Score = material::QUEEN_VALUE * 2 - material::PAWN_VALUE;

// ===== Null-move pruning =====

#[inline(always)]
pub fn null_move_reduction() -> u8 {
    value(idx::NULL_MOVE_REDUCTION) as u8
}
#[inline(always)]
pub fn null_move_min_depth() -> u8 {
    value(idx::NULL_MOVE_MIN_DEPTH) as u8
}

// ===== Late move reductions =====

/// Moves searched at full depth before reductions begin.
#[inline(always)]
pub fn lmr_full_depth_moves() -> usize {
    value(idx::LMR_FULL_DEPTH_MOVES) as usize
}
#[inline(always)]
pub fn lmr_min_depth() -> u8 {
    value(idx::LMR_MIN_DEPTH) as u8
}

/// Divisor in the reduction formula `1 + moves_searched / LMR_DIVISOR`.
///
/// Note this formula has **no depth term**. The standard
/// `log(depth) * log(moves)` table is a known gain, but swapping it in is a
/// tuning change and needs a match, not just a node count.
#[inline(always)]
pub fn lmr_divisor() -> usize {
    value(idx::LMR_DIVISOR) as usize
}

// ===== Aspiration windows =====

#[inline(always)]
pub fn aspiration_depth() -> u8 {
    value(idx::ASPIRATION_DEPTH) as u8
}
#[inline(always)]
pub fn aspiration_window() -> Score {
    value(idx::ASPIRATION_WINDOW)
}

/// Once the window has widened past this, give up and search the full range.
#[inline(always)]
pub fn aspiration_max_delta() -> Score {
    value(idx::ASPIRATION_MAX_DELTA)
}

// ===== Futility pruning =====

/// Indexed by depth, and clamped to its own length at the point of use rather
/// than bounded by a separate constant that could drift out of sync.
pub const FUTILITY_MARGIN: [Score; 4] = [0, 100, 200, 300];
#[inline(always)]
pub fn futility_max_depth() -> u8 {
    value(idx::FUTILITY_MAX_DEPTH) as u8
}

// ===== Reverse futility (static null move) =====

#[inline(always)]
pub fn rfp_margin() -> Score {
    value(idx::RFP_MARGIN)
}
#[inline(always)]
pub fn rfp_max_depth() -> u8 {
    value(idx::RFP_MAX_DEPTH) as u8
}

// ===== Miscellaneous =====

/// Plies of principal variation collected. Beyond this the PV is still searched,
/// just not recorded for reporting.
pub const PV_COLLECTION_LIMIT: usize = 32;

const _: () = assert!(
    TUNABLES[idx::FUTILITY_MAX_DEPTH].max as usize <= FUTILITY_MARGIN.len(),
    "futility margin table must cover every depth a tuner may select"
);

const _: () = assert!(
    TUNABLES[idx::LMR_DIVISOR].min >= 1,
    "LMR divisor must never reach zero"
);

#[cfg(test)]
mod tests {
    use super::*;

    /// Index constants and registry order must agree, or a tuner would move a
    /// different parameter than the one it named.
    #[test]
    fn test_registry_order_matches_index_constants() {
        assert_eq!(
            TUNABLES[idx::DELTA_PRUNING_MARGIN].name,
            "DeltaPruningMargin"
        );
        assert_eq!(TUNABLES[idx::NULL_MOVE_REDUCTION].name, "NullMoveReduction");
        assert_eq!(TUNABLES[idx::LMR_DIVISOR].name, "LmrDivisor");
        assert_eq!(TUNABLES[idx::RFP_MAX_DEPTH].name, "RfpMaxDepth");
        assert_eq!(TUNABLES.len(), idx::RFP_MAX_DEPTH + 1);
    }

    #[test]
    fn test_defaults_are_within_their_declared_range() {
        for t in TUNABLES {
            assert!(
                t.min <= t.default && t.default <= t.max,
                "{} out of range",
                t.name
            );
        }
    }

    /// Every accessor must report its declared default in a default build.
    #[test]
    fn test_accessors_report_defaults() {
        assert_eq!(delta_pruning_margin(), 200);
        assert_eq!(null_move_reduction(), 3);
        assert_eq!(lmr_divisor(), 6);
        assert_eq!(aspiration_window(), 25);
        assert_eq!(rfp_margin(), 120);
        assert_eq!(rfp_max_depth(), 3);
    }

    #[cfg(feature = "tune")]
    #[test]
    fn test_override_applies_and_clamps() {
        assert!(set_by_name("RfpMargin", 250));
        assert_eq!(rfp_margin(), 250);

        // Case-insensitive, as UCI option names are.
        assert!(set_by_name("rfpmargin", 300));
        assert_eq!(rfp_margin(), 300);

        // Out-of-range values are clamped, never applied raw: a zero divisor
        // or an out-of-bounds depth would take the search down.
        assert!(set_by_name("LmrDivisor", 0));
        assert_eq!(lmr_divisor(), 1);

        assert!(set_by_name("FutilityMaxDepth", 99));
        assert_eq!(futility_max_depth() as usize, FUTILITY_MARGIN.len() - 1);

        assert!(!set_by_name("NoSuchParameter", 1));

        // Leave the table as we found it, since these are process-global.
        assert!(set_by_name("RfpMargin", 120));
        assert!(set_by_name("LmrDivisor", 6));
        assert!(set_by_name("FutilityMaxDepth", 3));
    }
}
