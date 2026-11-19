//! Score-based merging after Dräger, Finkbeiner & Podelski (DFP), and its
//! variant that first merges within the strongly connected components of the
//! causal graph (SCC-DFP, Sievers, Wehrle & Helmert 2016).

use alloc::vec;
use alloc::vec::Vec;

use sas_plus::{CausalGraph, LabeledTransitionSystem, Task, VarId};

use crate::{Factor, MergeStrategy};

/// Merges the pair of factors whose shared labels lead closest to the goal.
///
/// The rank of a label in a factor is the smallest goal distance of a target
/// of one of its transitions, or infinite if the label has no transitions or
/// only self-loops on every state. A pair scores the smallest rank over labels
/// of the larger of the label's ranks in the two factors. Pairs with a factor
/// that has a non-goal state come first, then the lowest score, then the
/// lowest indices.
pub struct Dfp;

impl MergeStrategy for Dfp {
    fn next_pair(&mut self, factors: &[Option<Factor>], costs: &[u64]) -> (usize, usize) {
        let active: Vec<usize> = (0..factors.len())
            .filter(|&i| factors[i].is_some())
            .collect();

        best_pair(factors, &active, costs)
    }
}

/// Merges the factors of each strongly connected component of the causal
/// graph into one with [`Dfp`], components in topological order (sources
/// first), then merges the results with [`Dfp`].
pub struct SccDfp {
    /// Components with more than one variable, the next one last.
    sccs: Vec<Vec<VarId>>,
    /// Number of task variables.
    num_vars: usize,
}

impl SccDfp {
    /// Computes the strongly connected components of the causal graph of `task`.
    #[must_use]
    pub fn new(task: &Task) -> Self {
        let mut sccs = components(&CausalGraph::new(task), task.num_variables());
        sccs.retain(|c| c.len() > 1);
        sccs.reverse();

        Self {
            sccs,
            num_vars: task.num_variables(),
        }
    }
}

impl MergeStrategy for SccDfp {
    fn next_pair(&mut self, factors: &[Option<Factor>], costs: &[u64]) -> (usize, usize) {
        while let Some(scc) = self.sccs.last() {
            let mut inside = vec![false; self.num_vars];
            for &v in scc {
                inside[v] = true;
            }

            let candidates: Vec<usize> = (0..factors.len())
                .filter(|&i| {
                    factors[i]
                        .as_ref()
                        .is_some_and(|f| f.tree.variables().iter().all(|&v| inside[v]))
                })
                .collect();
            if candidates.len() > 1 {
                return best_pair(factors, &candidates, costs);
            }

            self.sccs.pop();
        }

        Dfp.next_pair(factors, costs)
    }
}

/// The best pair among the factors at indices `candidates` (at least two), by
/// goal relevance, then DFP score, then index.
fn best_pair(factors: &[Option<Factor>], candidates: &[usize], costs: &[u64]) -> (usize, usize) {
    let lts = |i: usize| &factors[i].as_ref().expect("candidates are active").lts;
    let ranks: Vec<Vec<Option<u64>>> = candidates
        .iter()
        .map(|&i| label_ranks(lts(i), costs))
        .collect();
    let relevant: Vec<bool> = candidates
        .iter()
        .map(|&i| lts(i).goal.iter().any(|&g| !g))
        .collect();

    let mut best = None;
    for x in 0..candidates.len() {
        for y in x + 1..candidates.len() {
            let score = ranks[x]
                .iter()
                .zip(&ranks[y])
                .filter_map(|(&a, &b)| Some(a?.max(b?)))
                .min();
            // `None` (infinite) sorts after every finite score.
            let key = (!(relevant[x] || relevant[y]), score.is_none(), score, x, y);
            if best.as_ref().is_none_or(|b| key < *b) {
                best = Some(key);
            }
        }
    }

    let (.., x, y) = best.expect("at least two candidates");
    (candidates[x], candidates[y])
}

/// The DFP rank of each label in `lts`: the smallest goal distance of a target
/// of one of its transitions; `None` for labels without transitions, with only
/// self-loops on every state, or whose targets are all dead ends.
fn label_ranks(lts: &LabeledTransitionSystem, costs: &[u64]) -> Vec<Option<u64>> {
    let h = lts.goal_distances(costs);
    let mut rank: Vec<Option<u64>> = vec![None; lts.num_labels];
    let mut self_loops = vec![0_usize; lts.num_labels];
    let mut moves = vec![false; lts.num_labels];
    for t in &lts.transitions {
        if t.src == t.dst {
            self_loops[t.label] += 1;
        } else {
            moves[t.label] = true;
        }
        if let Some(d) = h[t.dst] {
            rank[t.label] = Some(rank[t.label].map_or(d, |r| r.min(d)));
        }
    }

    for l in 0..lts.num_labels {
        if !moves[l] && self_loops[l] == lts.num_states {
            rank[l] = None;
        }
    }

    rank
}

/// Strongly connected components of the causal graph over `num_vars`
/// variables, in topological order (Kosaraju).
fn components(cg: &CausalGraph, num_vars: usize) -> Vec<Vec<VarId>> {
    let mut finished = Vec::with_capacity(num_vars);
    let mut seen = vec![false; num_vars];
    for v in 0..num_vars {
        postorder(v, |u| cg.successors(u), &mut seen, &mut finished);
    }

    let mut seen = vec![false; num_vars];
    let mut sccs = Vec::new();
    for &v in finished.iter().rev() {
        let mut scc = Vec::new();
        postorder(v, |u| cg.predecessors(u), &mut seen, &mut scc);
        if !scc.is_empty() {
            scc.sort_unstable();
            sccs.push(scc);
        }
    }

    sccs
}

/// Appends the unseen vertices reachable from `root` along `next` to `out`,
/// in depth-first postorder, and marks them seen.
fn postorder<'a>(
    root: VarId,
    next: impl Fn(VarId) -> &'a [VarId],
    seen: &mut [bool],
    out: &mut Vec<VarId>,
) {
    if core::mem::replace(&mut seen[root], true) {
        return;
    }

    let mut stack = vec![(root, 0)];
    while let Some((v, i)) = stack.pop() {
        match next(v).get(i) {
            Some(&u) => {
                stack.push((v, i + 1));
                if !core::mem::replace(&mut seen[u], true) {
                    stack.push((u, 0));
                }
            }
            None => out.push(v),
        }
    }
}
