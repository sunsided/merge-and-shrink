//! Fuzzes merge-and-shrink with exact bisimulation on random tasks against `h*`.
//!
//! The fuzzer picks the task (seed and size), the linear merge order and
//! whether to prune. Exact goal-respecting bisimulation preserves goal
//! distances, so the heuristic must equal `h*` on the initial state.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink};
use sas_plus::random::{RandomTaskParams, random_task};

/// One fuzz case: which task to generate, in which order to merge, and whether to prune.
#[derive(Arbitrary, Debug)]
struct Input {
    /// Seed for [`random_task`].
    seed: u64,
    /// Number of variables, taken modulo 5 plus one (1 to 5).
    num_vars: u8,
    /// Largest domain size, taken modulo 3 plus two (2 to 4).
    max_domain: u8,
    /// Number of operators, taken modulo 16 plus one (1 to 16).
    num_ops: u8,
    /// Sort keys for the variables; missing keys count as 0, ties keep index order.
    keys: Vec<u8>,
    /// Whether to drop unreachable and dead abstract states after each step.
    prune: bool,
}

/// Generator parameters bounded so that `h*` (a full projection) stays cheap.
fn params(input: &Input) -> RandomTaskParams {
    RandomTaskParams {
        num_vars: usize::from(input.num_vars % 5) + 1,
        max_domain: u32::from(input.max_domain % 3) + 2,
        num_ops: usize::from(input.num_ops % 16) + 1,
        ..RandomTaskParams::default()
    }
}

/// The variables `0..len` sorted by `keys`: a linear merge order.
fn order(keys: &[u8], len: usize) -> Vec<usize> {
    let key = |i: usize| keys.get(i).copied().unwrap_or(0);
    let mut order: Vec<usize> = (0..len).collect();
    order.sort_by_key(|&i| key(i));

    order
}

fuzz_target!(|input: Input| {
    let task = random_task(input.seed, params(&input));
    let merge = LinearMerge::new(order(&input.keys, task.num_variables()));

    let h = MergeAndShrink {
        merge,
        shrink: Bisimulation,
        prune: input.prune,
    }
    .build(&task);

    assert_eq!(
        h.value(&task.initial),
        task.perfect_heuristic()(&task.initial)
    );
});
