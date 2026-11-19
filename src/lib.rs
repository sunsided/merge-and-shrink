//! Merge-and-shrink abstractions (Helmert, Haslum, Hoffmann & Nissim 2014;
//! Sievers & Helmert 2021).
//!
//! Start from one atomic transition system per variable, then repeatedly
//! *merge* two factors into their synchronized product and *shrink* the result
//! by a state equivalence. Goal distances in the final factor are an
//! admissible heuristic. With exact bisimulation it is perfect, except on
//! tasks with conditional effects or derived variables, which atomic factors
//! over-approximate.
//!
//! Status: atomic factors, products, linear merging, pruning, and exact
//! goal-respecting bisimulation. Planned: label reduction, greedy/size-bounded
//! bisimulation, DFP/SCC-DFP merging, shrinking *before* merging.

#![no_std]

extern crate alloc;

mod factor;
mod merge;
mod shrink;

pub use factor::{Factor, MergeTree};
pub use merge::{LinearMerge, MergeStrategy};
pub use shrink::{Bisimulation, NoShrink, ShrinkStrategy};

use alloc::vec::Vec;

use sas_plus::{Abstraction, LabeledTransitionSystem, State, Task};

/// Builder for a merge-and-shrink heuristic: a merge strategy, a shrink strategy
/// and whether to prune.
pub struct MergeAndShrink<M, S> {
    /// Picks the next two factors to merge.
    pub merge: M,
    /// Shrinks each product after it is built and pruned.
    pub shrink: S,
    /// Remove unreachable and dead-end abstract states from each atomic factor,
    /// and from each product before it is shrunk.
    pub prune: bool,
}

impl<M: MergeStrategy, S: ShrinkStrategy> MergeAndShrink<M, S> {
    /// A builder with pruning enabled.
    #[must_use]
    pub fn new(merge: M, shrink: S) -> Self {
        Self {
            merge,
            shrink,
            prune: true,
        }
    }

    /// Merges the atomic factors of `task` into one and computes its goal distances.
    ///
    /// # Panics
    ///
    /// Panics if `task` has no variables, or if the merge strategy picks the same
    /// factor twice or a factor that was already merged.
    #[must_use]
    pub fn build(mut self, task: &Task) -> MergeAndShrinkHeuristic {
        let costs = task.costs();
        let mut factors = self.atomic_factors(task, &costs);

        for _ in 1..factors.len() {
            let product = self.merge_next(&mut factors, &costs);
            factors.push(Some(product));
        }

        let factor = factors
            .into_iter()
            .flatten()
            .next()
            .expect("task has at least one variable");
        let distances = factor.lts.goal_distances(&costs);

        MergeAndShrinkHeuristic { factor, distances }
    }

    /// One atomic factor per variable of `task`, pruned if enabled.
    fn atomic_factors(&self, task: &Task, costs: &[u64]) -> Vec<Option<Factor>> {
        (0..task.num_variables())
            .map(|v| {
                let mut f = Factor::atomic(task, v);
                if self.prune {
                    f.prune(costs);
                }

                Some(f)
            })
            .collect()
    }

    /// Takes the two factors the merge strategy picks, and returns their
    /// product, pruned and shrunk.
    fn merge_next(&mut self, factors: &mut [Option<Factor>], costs: &[u64]) -> Factor {
        let (i, j) = self.merge.next_pair(factors);
        assert_ne!(i, j, "merge strategy returned the same factor twice");
        let a = factors[i]
            .take()
            .expect("merge strategy chose an inactive factor");
        let b = factors[j]
            .take()
            .expect("merge strategy chose an inactive factor");

        let mut product = Factor::product(a, b);
        // Pruning first keeps dead and unreachable states out of shrinking. A quotient
        // of live states is live, so shrinking leaves nothing to prune afterwards
        // (see ShrinkStrategy::compute for what a strategy must not do).
        if self.prune {
            product.prune(costs);
        }

        if let Some((map, n)) = self.shrink.compute(&product, costs) {
            product.apply_abstraction(&map, n);
        }

        product
    }
}

/// The heuristic: goal distances in the final factor, looked up through its merge tree.
pub struct MergeAndShrinkHeuristic {
    /// The single factor left after merging every atomic factor.
    factor: Factor,
    /// Goal distance per abstract state of `factor`; `None` for dead ends.
    distances: Vec<Option<u64>>,
}

impl MergeAndShrinkHeuristic {
    /// Admissible estimate of the cost to the goal.
    ///
    /// `None` means `state` is a dead end, or, with pruning, a state that is
    /// unreachable from the task's initial state (even a goal state).
    #[must_use]
    pub fn value(&self, state: &State) -> Option<u64> {
        self.factor
            .tree
            .lookup(state)
            .and_then(|s| self.distances[s])
    }

    /// Number of states of the final factor: the size of the abstraction.
    #[must_use]
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
