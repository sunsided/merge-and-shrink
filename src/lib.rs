//! Merge-and-shrink abstractions (Helmert, Haslum, Hoffmann & Nissim 2014;
//! Sievers & Helmert 2021).
//!
//! Start from one atomic transition system per variable, then repeatedly
//! *shrink* two factors by a state equivalence and *merge* them into their
//! synchronized product. Goal distances in the final factor are an admissible
//! heuristic. With exact bisimulation and no size limit it is perfect, except
//! on tasks with conditional effects or derived variables, which atomic
//! factors over-approximate.
//!
//! Each step lets the [`MergeStrategy`] pick two factors ([`LinearMerge`],
//! [`Dfp`], [`SccDfp`]), optionally reduces labels, shrinks both with the
//! [`ShrinkStrategy`] so that their product fits [`MergeAndShrink::max_states`]
//! ([`Bisimulation`], [`GreedyBisimulation`], [`NoShrink`]), builds the
//! product and prunes it.

#![no_std]

extern crate alloc;

mod factor;
mod labels;
mod merge;
mod shrink;

pub use factor::{Factor, MergeTree};
pub use merge::{Dfp, LinearMerge, MergeStrategy, SccDfp};
pub use shrink::{Bisimulation, GreedyBisimulation, NoShrink, ShrinkStrategy};

use alloc::vec::Vec;

use labels::Labels;
use sas_plus::{Abstraction, LabeledTransitionSystem, State, Task};

/// Builder for a merge-and-shrink heuristic: a merge strategy, a shrink
/// strategy, a size limit and whether to prune.
pub struct MergeAndShrink<M, S> {
    /// Picks the next two factors to merge.
    pub merge: M,
    /// Shrinks the two factors of each merge right before the product is built.
    pub shrink: S,
    /// Remove unreachable and dead-end abstract states from each atomic factor
    /// and each product.
    pub prune: bool,
    /// Largest number of states a product may have; at least 1. Before each
    /// merge, the two factors are shrunk to sizes whose product stays within
    /// it. `usize::MAX` means no limit.
    pub max_states: usize,
    /// Before shrinking, combine labels (operators) of equal cost that only
    /// one factor can tell apart. This is exact and lets bisimulation shrink
    /// more; the final abstraction is labeled with operator ids again.
    pub label_reduction: bool,
}

impl<M: MergeStrategy, S: ShrinkStrategy> MergeAndShrink<M, S> {
    /// A builder with pruning and label reduction enabled, and no size limit.
    #[must_use]
    pub fn new(merge: M, shrink: S) -> Self {
        Self {
            merge,
            shrink,
            prune: true,
            max_states: usize::MAX,
            label_reduction: true,
        }
    }

    /// Merges the atomic factors of `task` into one and computes its goal distances.
    ///
    /// # Panics
    ///
    /// Panics if `task` has no variables, if `max_states` is 0, or if the merge
    /// strategy picks the same factor twice or a factor that was already merged.
    #[must_use]
    pub fn build(mut self, task: &Task) -> MergeAndShrinkHeuristic {
        assert!(self.max_states > 0, "max_states must be at least 1");
        let costs = task.costs();
        let mut factors = self.atomic_factors(task, &costs);
        let mut labels = Labels::new(costs.len());

        for _ in 1..factors.len() {
            let product = self.merge_next(&mut factors, &mut labels, &costs);
            factors.push(Some(product));
        }

        let mut factor = factors
            .into_iter()
            .flatten()
            .next()
            .expect("task has at least one variable");
        labels.expand(&mut factor.lts);
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

    /// Takes the two factors the merge strategy picks, reduces labels if
    /// enabled, shrinks the factors to fit the size limit, and returns their
    /// product, pruned.
    fn merge_next(
        &mut self,
        factors: &mut [Option<Factor>],
        labels: &mut Labels,
        costs: &[u64],
    ) -> Factor {
        let (i, j) = self.merge.next_pair(factors, costs);
        assert_ne!(i, j, "merge strategy returned the same factor twice");
        assert!(
            factors[i].is_some() && factors[j].is_some(),
            "merge strategy chose an inactive factor"
        );
        if self.label_reduction {
            labels.reduce(factors, i, costs);
            labels.reduce(factors, j, costs);
        }
        let mut a = factors[i]
            .take()
            .expect("merge strategy chose an inactive factor");
        let mut b = factors[j]
            .take()
            .expect("merge strategy chose an inactive factor");

        // Factors are pruned when built, so they hold only live states, and a
        // quotient of live states is live (see ShrinkStrategy::compute).
        let (max_a, max_b) = shrink_sizes(a.lts.num_states, b.lts.num_states, self.max_states);
        self.shrink_to(&mut a, costs, max_a);
        self.shrink_to(&mut b, costs, max_b);

        let mut product = Factor::product(a, b);
        if self.prune {
            product.prune(costs);
        }

        product
    }

    /// Shrinks `factor` with the shrink strategy, to at most `max_states` states.
    fn shrink_to(&self, factor: &mut Factor, costs: &[u64], max_states: usize) {
        if let Some((map, n)) = self.shrink.compute(factor, costs, max_states) {
            factor.apply_abstraction(&map, n);
        }
    }
}

/// Size limits for two factors of sizes `a` and `b` so that their product has
/// at most `max_states` states. Each keeps its size if the product fits;
/// otherwise the smaller one keeps as much as it can and the larger one gets
/// the rest, and if neither fits within `isqrt(max_states)`, both get about that.
fn shrink_sizes(a: usize, b: usize, max_states: usize) -> (usize, usize) {
    let (a, b) = (a.min(max_states), b.min(max_states));
    if a.saturating_mul(b) <= max_states {
        return (a, b);
    }

    let balanced = max_states.isqrt();
    if a <= balanced {
        (a, max_states / a)
    } else if b <= balanced {
        (max_states / b, b)
    } else {
        (balanced, max_states / balanced)
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
