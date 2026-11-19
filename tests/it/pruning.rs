//! What pruned states evaluate to.

use merge_and_shrink::{LinearMerge, MergeAndShrink, NoShrink};
use sas_plus::{Fact, State, Task, Variable};

/// With pruning, a goal state that is unreachable from the initial state
/// evaluates to `None`, like a dead end; without pruning, to its goal distance.
#[test]
fn pruning_maps_unreachable_goal_states_to_none() {
    // One variable, no operators: the initial state 0 cannot reach the goal value 1.
    let task = Task {
        variables: vec![Variable::new("v", 2)],
        operators: vec![],
        initial: State(vec![0]),
        goal: vec![Fact::new(0, 1)],
        axioms: vec![],
    };
    let build = |prune| {
        MergeAndShrink {
            merge: LinearMerge::in_variable_order(1),
            shrink: NoShrink,
            prune,
        }
        .build(&task)
    };
    let goal = State(vec![1]);

    assert_eq!(build(true).value(&goal), None);
    assert_eq!(build(false).value(&goal), Some(0));
}
