//! Properties of merge-and-shrink over random tasks, merge strategies and
//! label reduction settings.
//!
//! Each property compares against the exact heuristic `h*` from
//! [`sas_plus::Task::perfect_heuristic`]. `None` means a dead end.

use merge_and_shrink::{
    Bisimulation, GreedyBisimulation, MergeAndShrinkHeuristic, NoShrink, ShrinkStrategy,
};
use proptest::prelude::*;
use sas_plus::{Abstraction, Task, Transition};

use crate::strategies::{Case, case, rich_case, states_of};

/// Builds the heuristic for `case` without pruning or a size limit.
fn unpruned<S: ShrinkStrategy>(case: &Case, shrink: S) -> MergeAndShrinkHeuristic {
    case.build(shrink, false, usize::MAX)
}

/// Builds the heuristic for `case` with exact or greedy bisimulation, without
/// pruning, with at most `max_states` states per product.
fn bounded(case: &Case, greedy: bool, max_states: usize) -> MergeAndShrinkHeuristic {
    if greedy {
        case.build(GreedyBisimulation, false, max_states)
    } else {
        case.build(Bisimulation, false, max_states)
    }
}

/// Checks that `h` equals `h*` on every state of `task`.
fn assert_perfect(h: &MergeAndShrinkHeuristic, task: &Task) -> Result<(), TestCaseError> {
    let h_star = task.perfect_heuristic();
    for s in states_of(task) {
        prop_assert_eq!(h.value(&s), h_star(&s), "{:?}", s);
    }

    Ok(())
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
        assert_perfect(&unpruned(&case, NoShrink), &case.task)?;
    }

    /// Exact goal-respecting bisimulation preserves goal distances: h = h* on every state.
    #[test]
    fn bisimulation_is_perfect(case in case()) {
        assert_perfect(&unpruned(&case, Bisimulation), &case.task)?;
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
        let h = case.build(Bisimulation, true, usize::MAX);
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
        let h = bounded(&case, greedy, max_states);

        assert_admissible(&h, &case.task)?;
        assert_within(&h, &case.task, max_states)?;
    }

    /// The abstraction is labeled with operator ids, label reduction or not:
    /// every concrete transition `s -o-> t` has an abstract transition
    /// `α(s) -o-> α(t)`, as cost partitioning requires.
    #[test]
    fn abstract_transitions_cover_every_operator(
        case in case(),
        max_states in 1..=32_usize,
        greedy in any::<bool>(),
    ) {
        let h = bounded(&case, greedy, max_states);
        let lts = h.lts();
        for s in states_of(&case.task) {
            for op in case.task.applicable_operators(&s) {
                let t = case.task.apply(op, &s);
                let src = h.abstract_state(&s).expect("not pruned");
                let dst = h.abstract_state(&t).expect("not pruned");
                let transition = Transition { src, label: op, dst };

                prop_assert!(lts.transitions.contains(&transition), "{transition:?} missing");
            }
        }
    }
}
