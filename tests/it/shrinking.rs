//! Edge cases of the shrink strategies.

use merge_and_shrink::{Bisimulation, Factor, ShrinkStrategy};
use sas_plus::{Fact, State, Task, Variable};

/// Bisimulation leaves a factor without states alone, whatever the limit, and
/// rejects a limit of 0 for any other factor.
#[test]
fn bisimulation_needs_a_positive_limit_only_for_nonempty_factors() {
    let task = Task {
        variables: vec![Variable::new("v0", 2)],
        operators: vec![],
        initial: State(vec![0]),
        goal: vec![Fact::new(0, 1)],
        axioms: vec![],
    };
    let mut factor = Factor::atomic(&task, 0);
    let costs = task.costs();
    let limit_zero = std::panic::catch_unwind(|| Bisimulation.compute(&factor, &costs, 0));
    assert!(limit_zero.is_err());

    factor.apply_abstraction(&[None, None], 0);
    assert_eq!(Bisimulation.compute(&factor, &costs, 0), None);
}
