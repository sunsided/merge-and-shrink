//! proptest strategies and shared helpers: random planning tasks with a merge
//! strategy and a label reduction setting.

use merge_and_shrink::{
    Dfp, Factor, LinearMerge, MergeAndShrink, MergeAndShrinkHeuristic, MergeStrategy, SccDfp,
    ShrinkStrategy,
};
use proptest::prelude::*;
use sas_plus::random::{RandomTaskParams, random_task};
use sas_plus::{State, Task};

/// Which merge strategy a [`Case`] uses.
#[derive(Clone, Copy, Debug)]
pub enum MergeKind {
    /// [`LinearMerge`] in the case's `order`.
    Linear,
    /// [`Dfp`].
    Dfp,
    /// [`SccDfp`].
    SccDfp,
}

/// A random task, a merge strategy and whether to reduce labels.
#[derive(Debug)]
pub struct Case {
    /// The planning task; generated from a seed by [`random_task`].
    pub task: Task,
    /// A permutation of `0..task.num_variables()`, the order a linear merge uses.
    pub order: Vec<usize>,
    /// The merge strategy.
    pub merge: MergeKind,
    /// Whether to reduce labels.
    pub label_reduction: bool,
}

impl Case {
    /// Builds the heuristic with `shrink`, at most `max_states` states per
    /// product, and pruning if `prune`.
    pub fn build<S: ShrinkStrategy>(
        &self,
        shrink: S,
        prune: bool,
        max_states: usize,
    ) -> MergeAndShrinkHeuristic {
        let merge = match self.merge {
            MergeKind::Linear => AnyMerge::Linear(LinearMerge::new(self.order.clone())),
            MergeKind::Dfp => AnyMerge::Dfp(Dfp),
            MergeKind::SccDfp => AnyMerge::SccDfp(SccDfp::new(&self.task)),
        };

        MergeAndShrink {
            merge,
            shrink,
            prune,
            max_states,
            label_reduction: self.label_reduction,
        }
        .build(&self.task)
    }
}

/// One of the crate's merge strategies, chosen at run time.
enum AnyMerge {
    /// A linear merge.
    Linear(LinearMerge),
    /// DFP.
    Dfp(Dfp),
    /// SCC-DFP.
    SccDfp(SccDfp),
}

impl MergeStrategy for AnyMerge {
    fn next_pair(&mut self, factors: &[Option<Factor>], costs: &[u64]) -> (usize, usize) {
        match self {
            Self::Linear(m) => m.next_pair(factors, costs),
            Self::Dfp(m) => m.next_pair(factors, costs),
            Self::SccDfp(m) => m.next_pair(factors, costs),
        }
    }
}

/// Random plain SAS+ tasks small enough to enumerate every state, with a
/// random merge strategy, a shuffled linear merge order and a random label
/// reduction setting.
///
/// Up to 4 variables with up to 3 values each keep the state space at 81 states
/// at most, so checking every state stays cheap.
pub fn case() -> impl Strategy<Value = Case> {
    cases(false)
}

/// Like [`case`], but with conditional effects and up to two derived variables.
///
/// Atomic factors over-approximate both features, so the heuristic is
/// admissible on these tasks but not perfect.
pub fn rich_case() -> impl Strategy<Value = Case> {
    cases(true)
}

/// Random cases; `rich` adds conditional effects and derived variables.
fn cases(rich: bool) -> impl Strategy<Value = Case> {
    (any::<u64>(), 1..=4_usize, 2..=3_u32, 1..=12_usize).prop_flat_map(
        move |(seed, num_vars, max_domain, num_ops)| {
            let params = RandomTaskParams {
                num_vars,
                max_domain,
                num_ops,
                max_conditions: if rich { 2 } else { 0 },
                num_derived: if rich { 2 } else { 0 },
                ..RandomTaskParams::default()
            };
            let task = random_task(seed, params);
            let identity: Vec<usize> = (0..task.num_variables()).collect();

            let merge = prop_oneof![
                Just(MergeKind::Linear),
                Just(MergeKind::Dfp),
                Just(MergeKind::SccDfp)
            ];

            (
                Just(task),
                Just(identity).prop_shuffle(),
                merge,
                any::<bool>(),
            )
                .prop_map(|(task, order, merge, label_reduction)| Case {
                    task,
                    order,
                    merge,
                    label_reduction,
                })
        },
    )
}

/// Every state of a task with the given variable domain sizes, in lexicographic order.
fn all_states(domains: &[u32]) -> Vec<State> {
    let mut states = vec![State(vec![])];
    for &d in domains {
        states = states
            .into_iter()
            .flat_map(|s| {
                (0..d).map(move |v| {
                    let mut s = s.clone();
                    s.0.push(v);
                    s
                })
            })
            .collect();
    }

    states
}

/// Every state of `task`.
pub fn states_of(task: &Task) -> Vec<State> {
    let domains: Vec<u32> = task.variables.iter().map(|v| v.domain_size).collect();

    all_states(&domains)
}
