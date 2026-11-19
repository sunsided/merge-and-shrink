use std::collections::HashMap;

use crate::Factor;

/// Computes a state equivalence for a factor: `Some((map, num_classes))`, or
/// `None` to leave it unchanged.
pub trait ShrinkStrategy {
    fn compute(&self, factor: &Factor, costs: &[u64]) -> Option<(Vec<Option<usize>>, usize)>;
}

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
        let mut out: Vec<Vec<(usize, usize)>> = vec![Vec::new(); lts.num_states];
        for t in &lts.transitions {
            out[t.src].push((t.label, t.dst));
        }

        let mut block = assign_ids((0..lts.num_states).map(|s| (lts.goal[s], h[s])));
        let mut num_blocks = block.iter().max().map_or(0, |m| m + 1);
        loop {
            let signatures = (0..lts.num_states).map(|s| {
                let mut sig: Vec<(usize, usize)> =
                    out[s].iter().map(|&(l, d)| (l, block[d])).collect();
                sig.sort_unstable();
                sig.dedup();
                (block[s], sig)
            });
            let refined = assign_ids(signatures);
            let n = refined.iter().max().map_or(0, |m| m + 1);
            block = refined;
            if n == num_blocks {
                break;
            }
            num_blocks = n;
        }
        (num_blocks < lts.num_states).then(|| (block.into_iter().map(Some).collect(), num_blocks))
    }
}

fn assign_ids<K: std::hash::Hash + Eq>(keys: impl Iterator<Item = K>) -> Vec<usize> {
    let mut ids = HashMap::new();
    keys.map(|k| {
        let n = ids.len();
        *ids.entry(k).or_insert(n)
    })
    .collect()
}
