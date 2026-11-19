//! Merge-and-shrink abstractions (Helmert, Haslum, Hoffmann & Nissim 2014;
//! Sievers & Helmert 2021).
//!
//! Start from one atomic transition system per variable, then repeatedly
//! *merge* two factors into their synchronized product and *shrink* the result
//! by a state equivalence. Goal distances in the final factor are an
//! admissible (and, with exact bisimulation, perfect) heuristic.
//!
//! Status: atomic factors, products, linear merging, pruning, and exact
//! goal-respecting bisimulation. Planned: label reduction, greedy/size-bounded
//! bisimulation, DFP/SCC-DFP merging, shrinking *before* merging.

mod factor;
mod merge;
mod shrink;

pub use factor::{Factor, MergeTree};
pub use merge::{LinearMerge, MergeStrategy};
pub use shrink::{Bisimulation, NoShrink, ShrinkStrategy};

use sas_plus::{Abstraction, LabeledTransitionSystem, State, Task};

pub struct MergeAndShrink<M, S> {
    pub merge: M,
    pub shrink: S,
    /// Remove unreachable and dead-end abstract states after each step.
    pub prune: bool,
}

impl<M: MergeStrategy, S: ShrinkStrategy> MergeAndShrink<M, S> {
    pub fn new(merge: M, shrink: S) -> Self {
        Self {
            merge,
            shrink,
            prune: true,
        }
    }

    pub fn build(mut self, task: &Task) -> MergeAndShrinkHeuristic {
        let costs = task.costs();
        let mut factors: Vec<Option<Factor>> = (0..task.num_variables())
            .map(|v| {
                let mut f = Factor::atomic(task, v);
                if self.prune {
                    f.prune(&costs);
                }
                Some(f)
            })
            .collect();
        let mut remaining = factors.len();
        while remaining > 1 {
            let (i, j) = self.merge.next_pair(&factors);
            assert_ne!(i, j, "merge strategy returned the same factor twice");
            let a = factors[i]
                .take()
                .expect("merge strategy chose an inactive factor");
            let b = factors[j]
                .take()
                .expect("merge strategy chose an inactive factor");
            let mut product = Factor::product(a, b);
            if let Some((map, n)) = self.shrink.compute(&product, &costs) {
                product.apply_abstraction(&map, n);
            }
            if self.prune {
                product.prune(&costs);
            }
            factors.push(Some(product));
            remaining -= 1;
        }
        let factor = factors
            .into_iter()
            .flatten()
            .next()
            .expect("task has at least one variable");
        let distances = factor.lts.goal_distances(&costs);
        MergeAndShrinkHeuristic { factor, distances }
    }
}

pub struct MergeAndShrinkHeuristic {
    factor: Factor,
    distances: Vec<Option<u64>>,
}

impl MergeAndShrinkHeuristic {
    /// Admissible estimate of the cost to the goal; `None` = dead end.
    pub fn value(&self, state: &State) -> Option<u64> {
        self.factor
            .tree
            .lookup(state)
            .and_then(|s| self.distances[s])
    }

    pub fn num_abstract_states(&self) -> usize {
        self.factor.lts.num_states
    }
}

impl Abstraction for MergeAndShrinkHeuristic {
    fn lts(&self) -> &LabeledTransitionSystem {
        &self.factor.lts
    }
    fn abstract_state(&self, state: &State) -> Option<usize> {
        self.factor.tree.lookup(state)
    }
}

/// Doctests for the code examples in README.md (not part of the API).
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
