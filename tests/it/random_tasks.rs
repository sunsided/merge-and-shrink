//! Fixed seeds of random tasks, checked against `h*` on every state.

use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink, NoShrink};
use sas_plus::random::{RandomTaskParams, random_task};

use crate::strategies::states_of;

/// Without shrinking, the final factor is the full state space: h = h*.
/// Exact bisimulation preserves goal distances: still h = h*, but smaller.
#[test]
fn unshrunk_and_bisimulation_are_perfect() {
    for seed in 0..150 {
        let task = random_task(seed, RandomTaskParams::default());
        let h_star = task.perfect_heuristic();
        let n = task.num_variables();

        let full = MergeAndShrink {
            merge: LinearMerge::in_variable_order(n),
            shrink: NoShrink,
            prune: false,
        }
        .build(&task);
        let bisim = MergeAndShrink {
            merge: LinearMerge::in_variable_order(n),
            shrink: Bisimulation,
            prune: false,
        }
        .build(&task);
        assert!(bisim.num_abstract_states() <= full.num_abstract_states());

        for s in states_of(&task) {
            assert_eq!(full.value(&s), h_star(&s), "seed {seed}, no shrink, {s:?}");
            assert_eq!(
                bisim.value(&s),
                h_star(&s),
                "seed {seed}, bisimulation, {s:?}"
            );
        }

        // With pruning, values are only guaranteed on reachable states.
        let pruned = MergeAndShrink::new(LinearMerge::new((0..n).rev().collect()), Bisimulation)
            .build(&task);
        assert_eq!(
            pruned.value(&task.initial),
            h_star(&task.initial),
            "seed {seed}, pruned"
        );
    }
}
