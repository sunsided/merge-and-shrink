//! Properties of merge-and-shrink over random tasks and random linear merge orders.
//!
//! Each property compares against the exact heuristic `h*` from
//! [`sas_plus::Task::perfect_heuristic`]. `None` means a dead end.

use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink, NoShrink, ShrinkStrategy};
use proptest::prelude::*;

use crate::strategies::{Case, case, rich_case, states_of};

/// Builds the heuristic for `case` in its merge order, without pruning.
fn unpruned<S: ShrinkStrategy>(
    case: &Case,
    shrink: S,
) -> merge_and_shrink::MergeAndShrinkHeuristic {
    MergeAndShrink {
        merge: LinearMerge::new(case.order.clone()),
        shrink,
        prune: false,
    }
    .build(&case.task)
}

proptest! {
    /// Without shrinking, the final factor is the full state space: h = h* on every state.
    #[test]
    fn no_shrink_is_perfect(case in case()) {
        let h = unpruned(&case, NoShrink);
        let h_star = case.task.perfect_heuristic();

        for s in states_of(&case.task) {
            prop_assert_eq!(h.value(&s), h_star(&s), "{:?}", s);
        }
    }

    /// Exact goal-respecting bisimulation preserves goal distances: h = h* on every state.
    #[test]
    fn bisimulation_is_perfect(case in case()) {
        let h = unpruned(&case, Bisimulation);
        let h_star = case.task.perfect_heuristic();

        for s in states_of(&case.task) {
            prop_assert_eq!(h.value(&s), h_star(&s), "{:?}", s);
        }
    }

    /// Bisimulation never has more abstract states than the unshrunk product.
    #[test]
    fn bisimulation_is_no_larger_than_the_product(case in case()) {
        let full = unpruned(&case, NoShrink).num_abstract_states();
        let bisim = unpruned(&case, Bisimulation).num_abstract_states();

        prop_assert!(bisim <= full, "{bisim} > {full}");
    }

    /// Pruning drops only unreachable and dead states, so h* survives on the initial state.
    #[test]
    fn pruned_bisimulation_is_perfect_on_the_initial_state(case in case()) {
        let h = MergeAndShrink::new(LinearMerge::new(case.order.clone()), Bisimulation)
            .build(&case.task);
        let h_star = case.task.perfect_heuristic();

        prop_assert_eq!(h.value(&case.task.initial), h_star(&case.task.initial));
    }

    /// With conditional effects and derived variables, exact bisimulation stays
    /// admissible on every state and never claims a false dead end.
    #[test]
    fn bisimulation_is_admissible_with_conditional_effects_and_axioms(case in rich_case()) {
        let h = unpruned(&case, Bisimulation);
        let h_star = case.task.perfect_heuristic();

        for s in states_of(&case.task) {
            match (h.value(&s), h_star(&s)) {
                (Some(h), Some(opt)) => prop_assert!(h <= opt, "{h} > h* {opt} in {s:?}"),
                (None, opt) => prop_assert_eq!(opt, None, "false dead end in {:?}", s),
                (Some(_), None) => {}
            }
        }
    }
}
