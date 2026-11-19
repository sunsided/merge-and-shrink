//! Shrink strategies: state equivalences that make a factor smaller.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use sas_plus::LabeledTransitionSystem;

use crate::Factor;

/// Computes a state equivalence for a factor: `Some((map, num_classes))`, or
/// `None` to leave it unchanged.
pub trait ShrinkStrategy {
    /// The equivalence for `factor` under per-label `costs`, with at most
    /// `max_states` classes.
    ///
    /// `map` has one entry per state of `factor.lts`, with class ids in
    /// `0..num_classes`; `None` entries prune the state. `max_states` is at
    /// least 1, except for a factor without states. A strategy that returns more classes lets the product it feeds
    /// exceed [`MergeAndShrink::max_states`](crate::MergeAndShrink::max_states).
    ///
    /// The factor is shrunk right before it is merged. With pruning, it has
    /// only live states (reachable and able to reach a goal). Merging states
    /// keeps every class live. Unused class ids leave empty classes, which keep
    /// the heuristic admissible but make the factor larger. A `None` entry
    /// deletes a state: its concrete states evaluate to `None`, which is
    /// inadmissible if they can reach a goal, and states whose paths ran
    /// through it can become dead ends that stay in the product until it is
    /// pruned.
    fn compute(
        &self,
        factor: &Factor,
        costs: &[u64],
        max_states: usize,
    ) -> Option<(Vec<Option<usize>>, usize)>;
}

/// Never shrinks, and ignores the size limit. Without pruning, the final factor
/// is the full product and the heuristic is perfect on every state; with
/// pruning, on every reachable state.
pub struct NoShrink;

impl ShrinkStrategy for NoShrink {
    fn compute(&self, _: &Factor, _: &[u64], _: usize) -> Option<(Vec<Option<usize>>, usize)> {
        None
    }
}

/// Coarsest goal-respecting bisimulation, by signature refinement.
///
/// Within the size limit it preserves goal distances exactly, and the result
/// composes with later merges, so the final heuristic is perfect. At the limit
/// it stops refining: blocks are split in order of increasing goal distance
/// until a split would exceed the limit (Fast Downward's `at_limit=return`). If the initial blocks
/// (states with equal goal distance and goal status) already exceed it, those
/// with the largest goal distances are combined. Either way the heuristic stays
/// admissible.
pub struct Bisimulation;

impl ShrinkStrategy for Bisimulation {
    fn compute(
        &self,
        factor: &Factor,
        costs: &[u64],
        max_states: usize,
    ) -> Option<(Vec<Option<usize>>, usize)> {
        bisimulation(&factor.lts, costs, max_states, false)
    }
}

/// Greedy bisimulation (Nissim, Hoffmann & Helmert 2011): like
/// [`Bisimulation`], but signatures only contain transitions that start an
/// optimal path to the goal (`h(s) = c(l) + h(t)`).
///
/// It preserves the goal distances of the factor it shrinks, so it is coarser
/// than exact bisimulation at no cost there, but it does not compose with later
/// merges: the final heuristic is admissible, not perfect.
pub struct GreedyBisimulation;

impl ShrinkStrategy for GreedyBisimulation {
    fn compute(
        &self,
        factor: &Factor,
        costs: &[u64],
        max_states: usize,
    ) -> Option<(Vec<Option<usize>>, usize)> {
        bisimulation(&factor.lts, costs, max_states, true)
    }
}

/// Signature refinement from the goal-distance partition, with at most
/// `max_states` blocks. `greedy` keeps only transitions on optimal paths.
fn bisimulation(
    lts: &LabeledTransitionSystem,
    costs: &[u64],
    max_states: usize,
    greedy: bool,
) -> Option<(Vec<Option<usize>>, usize)> {
    let h = lts.goal_distances(costs);
    let out = outgoing(lts, costs, &h, greedy);

    // Block ids are ordered by goal distance (dead ends last), and refinement
    // keeps that order, so splitting low ids first refines near the goal first.
    let (mut block, mut num_blocks) =
        assign_ids((0..lts.num_states).map(|s| (h[s].is_none(), h[s], lts.goal[s])));
    if num_blocks > max_states {
        for b in &mut block {
            *b = (*b).min(max_states - 1);
        }
        num_blocks = max_states;
    }

    loop {
        let (refined, n, at_limit) = refine(&block, num_blocks, &out, max_states);
        let done = n == num_blocks || at_limit;
        block = refined;
        num_blocks = n;
        if done {
            break;
        }
    }

    (num_blocks < lts.num_states).then(|| (block.into_iter().map(Some).collect(), num_blocks))
}

/// Outgoing `(label, target)` pairs per state of `lts`. With `greedy`, only
/// transitions with `h(src) = cost(label) + h(dst)`.
fn outgoing(
    lts: &LabeledTransitionSystem,
    costs: &[u64],
    h: &[Option<u64>],
    greedy: bool,
) -> Vec<Vec<(usize, usize)>> {
    let mut out: Vec<Vec<(usize, usize)>> = vec![Vec::new(); lts.num_states];
    for t in &lts.transitions {
        let optimal = h[t.src].is_some() && h[t.src] == h[t.dst].map(|d| d + costs[t.label]);
        if !greedy || optimal {
            out[t.src].push((t.label, t.dst));
        }
    }

    out
}

/// One signature-refinement round: splits blocks by the set of
/// `(label, target block)` pairs their states can reach in one step, lowest
/// block ids first, until a split would exceed `max_states`. Returns the new
/// blocks, their count (unchanged when nothing split) and whether the limit
/// stopped the round.
fn refine(
    block: &[usize],
    num_blocks: usize,
    out: &[Vec<(usize, usize)>],
    max_states: usize,
) -> (Vec<usize>, usize, bool) {
    let signatures = out.iter().enumerate().map(|(s, edges)| {
        let mut sig: Vec<(usize, usize)> = edges.iter().map(|&(l, d)| (l, block[d])).collect();
        sig.sort_unstable();
        sig.dedup();

        (block[s], sig)
    });
    let (sub, num_sub) = assign_ids(signatures);
    if num_sub <= max_states {
        return (sub, num_sub, false);
    }

    // Too many: accept splits block by block until one does not fit.
    let mut parts = vec![0_usize; num_blocks];
    let mut seen = vec![false; num_sub];
    for (s, &b) in block.iter().enumerate() {
        if !core::mem::replace(&mut seen[sub[s]], true) {
            parts[b] += 1;
        }
    }

    let mut total = num_blocks;
    let mut split = vec![false; num_blocks];
    for (b, &p) in parts.iter().enumerate() {
        if total + p - 1 > max_states {
            break;
        }
        total += p - 1;
        split[b] = true;
    }

    let (refined, n) = assign_ids(
        block
            .iter()
            .zip(&sub)
            .map(|(&b, &s)| (b, if split[b] { s } else { 0 })),
    );
    (refined, n, true)
}

/// Numbers keys densely in key order: equal keys get equal ids. Returns the ids
/// and the number of distinct keys.
pub(crate) fn assign_ids<K: Ord + Clone>(keys: impl Iterator<Item = K>) -> (Vec<usize>, usize) {
    let keys: Vec<K> = keys.collect();
    let mut ids: BTreeMap<K, usize> = keys.iter().cloned().map(|k| (k, 0)).collect();
    for (n, id) in ids.values_mut().enumerate() {
        *id = n;
    }

    (keys.iter().map(|k| ids[k]).collect(), ids.len())
}
