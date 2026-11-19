//! What pruned states evaluate to.

use merge_and_shrink::{LinearMerge, MergeAndShrink, NoShrink};
use sas_plus::{Effect, Fact, Operator, State, Task, Variable};

/// With pruning, a goal state that is unreachable from the initial state
/// evaluates to `None`, like a dead end, while reachable states keep their
/// goal distances; without pruning, it evaluates to its goal distance.
#[test]
fn pruning_maps_unreachable_goal_states_to_none() {
    // The goal is v0 = 1. The only operator sets v0 to 1 while v1 = 0, and
    // nothing changes v1, so from (0, 0) the goal state (1, 1) is unreachable.
    let task = Task {
        variables: vec![Variable::new("v0", 2), Variable::new("v1", 2)],
        operators: vec![Operator {
            name: "advance".into(),
            pre: vec![Fact::new(1, 0)],
            eff: vec![Effect::new(Fact::new(0, 1))],
            cost: 1,
        }],
        initial: State(vec![0, 0]),
        goal: vec![Fact::new(0, 1)],
        axioms: vec![],
    };
    let build = |prune| {
        MergeAndShrink {
            merge: LinearMerge::in_variable_order(2),
            shrink: NoShrink,
            prune,
            max_states: usize::MAX,
        }
        .build(&task)
    };
    let unreachable_goal = State(vec![1, 1]);

    let pruned = build(true);
    assert!(pruned.num_abstract_states() > 0);
    assert_eq!(pruned.value(&task.initial), Some(1));
    assert_eq!(pruned.value(&State(vec![1, 0])), Some(0));
    assert_eq!(pruned.value(&unreachable_goal), None);

    assert_eq!(build(false).value(&unreachable_goal), Some(0));
}
