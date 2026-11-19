//! Properties of merge-and-shrink over random tasks and random linear merge orders.
//!
//! Each property compares against the exact heuristic `h*` from
//! [`sas_plus::Task::perfect_heuristic`]. `None` means a dead end.

use merge_and_shrink::{
    Bisimulation, GreedyBisimulation, LinearMerge, MergeAndShrink, MergeAndShrinkHeuristic,
    NoShrink, ShrinkStrategy,
};
use proptest::prelude::*;
use sas_plus::Task;

use crate::strategies::{Case, case, rich_case, states_of};

/// Builds the heuristic for `case` in its merge order, without pruning or a size limit.
fn unpruned<S: ShrinkStrategy>(case: &Case, shrink: S) -> MergeAndShrinkHeuristic {
    bounded(case, shrink, usize::MAX)
}

/// Builds the heuristic for `case` in its merge order, without pruning, with
/// at most `max_states` states per product.
fn bounded<S: ShrinkStrategy>(
    case: &Case,
    shrink: S,
    max_states: usize,
) -> MergeAndShrinkHeuristic {
    MergeAndShrink {
        merge: LinearMerge::new(case.order.clone()),
        shrink,
        prune: false,
        max_states,
    }
    .build(&case.task)
}

/// Checks that `h` never exceeds `h*` on any state of `task` and never claims a
/// false dead end.
fn assert_admissible(h: &MergeAndShrinkHeuristic, task: &Task) -> Result<(), TestCaseError> {
    let h_star = task.perfect_heuristic();
    for s in states_of(task) {
        match (h.value(&s), h_star(&s)) {
            (Some(h), Some(opt)) => prop_assert!(h <= opt, "{h} > h* {opt} in {s:?}"),
            (None, opt) => prop_assert_eq!(opt, None, "false dead end in {:?}", s),
            (Some(_), None) => {}
        }
    }

    Ok(())
}

/// Checks that the final factor of `h` respects `max_states`. A task with one
/// variable is never merged, so its atomic factor is never shrunk.
fn assert_within(
    h: &MergeAndShrinkHeuristic,
    task: &Task,
    max_states: usize,
) -> Result<(), TestCaseError> {
    if task.num_variables() > 1 {
        prop_assert!(
            h.num_abstract_states() <= max_states,
            "{} > {max_states}",
            h.num_abstract_states()
        );
    }

    Ok(())
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
        assert_admissible(&unpruned(&case, Bisimulation), &case.task)?;
    }

    /// Greedy bisimulation is admissible but, across merges, not perfect.
    #[test]
    fn greedy_bisimulation_is_admissible(case in case()) {
        assert_admissible(&unpruned(&case, GreedyBisimulation), &case.task)?;
    }

    /// Size-bounded bisimulation, exact or greedy, stays admissible and within the limit.
    #[test]
    fn bounded_bisimulation_is_admissible_and_within_the_limit(
        case in rich_case(),
        max_states in 1..=16_usize,
        greedy in any::<bool>(),
    ) {
        let h = if greedy {
            bounded(&case, GreedyBisimulation, max_states)
        } else {
            bounded(&case, Bisimulation, max_states)
        };

        assert_admissible(&h, &case.task)?;
        assert_within(&h, &case.task, max_states)?;
    }
}
