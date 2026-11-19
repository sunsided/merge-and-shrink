//! proptest strategies and shared helpers: random planning tasks with a merge order.

use proptest::prelude::*;
use sas_plus::random::{RandomTaskParams, random_task};
use sas_plus::{State, Task};

/// A random task and a linear merge order over its variables.
#[derive(Debug)]
pub struct Case {
    /// The planning task; generated from a seed by [`random_task`].
    pub task: Task,
    /// A permutation of `0..task.num_variables()`, the order factors are merged in.
    pub order: Vec<usize>,
}

/// Random tasks small enough to enumerate every state, with a shuffled merge order.
///
/// Up to 4 variables with up to 3 values each keep the state space at 81 states
/// at most, so checking every state stays cheap.
pub fn case() -> impl Strategy<Value = Case> {
    (any::<u64>(), 1..=4_usize, 2..=3_u32, 1..=12_usize).prop_flat_map(
        |(seed, num_vars, max_domain, num_ops)| {
            let params = RandomTaskParams {
                num_vars,
                max_domain,
                num_ops,
                ..RandomTaskParams::default()
            };
            let task = random_task(seed, params);
            let identity: Vec<usize> = (0..task.num_variables()).collect();

            (Just(task), Just(identity).prop_shuffle())
                .prop_map(|(task, order)| Case { task, order })
        },
    )
}

/// Every state of a task with the given variable domain sizes, in lexicographic order.
pub fn all_states(domains: &[u32]) -> Vec<State> {
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
