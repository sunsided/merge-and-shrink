//! Shrink strategies: state equivalences that make a factor smaller.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use sas_plus::LabeledTransitionSystem;

use crate::Factor;

/// Computes a state equivalence for a factor: `Some((map, num_classes))`, or
/// `None` to leave it unchanged.
pub trait ShrinkStrategy {
    /// The equivalence for `factor` under per-label `costs`.
    ///
    /// `map` has one entry per state of `factor.lts`, with class ids in
    /// `0..num_classes`; `None` entries prune the state.
    fn compute(&self, factor: &Factor, costs: &[u64]) -> Option<(Vec<Option<usize>>, usize)>;
}

/// Never shrinks. Without pruning, the final factor is the full product and the
/// heuristic is perfect on every state; with pruning, on every reachable state.
pub struct NoShrink;

impl ShrinkStrategy for NoShrink {
    fn compute(&self, _: &Factor, _: &[u64]) -> Option<(Vec<Option<usize>>, usize)> {
        None
    }
}

/// Coarsest goal-respecting bisimulation, by signature refinement.
/// Preserves goal distances exactly, so shrinking with it keeps the final
/// heuristic perfect.
pub struct Bisimulation;

impl ShrinkStrategy for Bisimulation {
    fn compute(&self, factor: &Factor, costs: &[u64]) -> Option<(Vec<Option<usize>>, usize)> {
        let lts = &factor.lts;
        let h = lts.goal_distances(costs);
        let out = outgoing(lts);

        let mut block = assign_ids((0..lts.num_states).map(|s| (lts.goal[s], h[s])));
        let mut num_blocks = count_ids(&block);
        loop {
            let refined = refine(&block, &out);
            let n = count_ids(&refined);
            block = refined;
            if n == num_blocks {
                break;
            }

            num_blocks = n;
        }

        (num_blocks < lts.num_states).then(|| (block.into_iter().map(Some).collect(), num_blocks))
    }
}

/// Outgoing `(label, target)` pairs per state of `lts`.
fn outgoing(lts: &LabeledTransitionSystem) -> Vec<Vec<(usize, usize)>> {
    let mut out: Vec<Vec<(usize, usize)>> = vec![Vec::new(); lts.num_states];
    for t in &lts.transitions {
        out[t.src].push((t.label, t.dst));
    }

    out
}

/// One signature-refinement round: splits each block by the set of
/// `(label, target block)` pairs its states can reach in one step.
fn refine(block: &[usize], out: &[Vec<(usize, usize)>]) -> Vec<usize> {
    let signatures = out.iter().enumerate().map(|(s, edges)| {
        let mut sig: Vec<(usize, usize)> = edges.iter().map(|&(l, d)| (l, block[d])).collect();
        sig.sort_unstable();
        sig.dedup();

        (block[s], sig)
    });

    assign_ids(signatures)
}

/// Number of distinct ids produced by [`assign_ids`]: the largest id plus one.
fn count_ids(ids: &[usize]) -> usize {
    ids.iter().max().map_or(0, |m| m + 1)
}

/// Numbers keys densely in order of first appearance: equal keys get equal ids.
fn assign_ids<K: Ord>(keys: impl Iterator<Item = K>) -> Vec<usize> {
    let mut ids = BTreeMap::new();
    keys.map(|k| {
        let n = ids.len();
        *ids.entry(k).or_insert(n)
    })
    .collect()
}
