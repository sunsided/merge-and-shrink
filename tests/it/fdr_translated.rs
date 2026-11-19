//! Fast Downward translator output with known optimal costs.

use merge_and_shrink::{
    Bisimulation, Dfp, LinearMerge, MergeAndShrink, MergeAndShrinkHeuristic, MergeStrategy, SccDfp,
};
use sas_plus::Task;
use sas_plus::fdr::SasFile;

/// The translated fixtures and their optimal plan costs.
const FIXTURES: [(&str, u64); 3] = [("sussman-anomaly", 3), ("gripper-2", 3), ("gripper-4", 9)];

/// Reads the translated fixture `name`.
fn fixture(name: &str) -> Task {
    let path = format!(
        "{}/tests/fixtures/translated/{name}/output.sas",
        env!("CARGO_MANIFEST_DIR")
    );

    SasFile::parse(&std::fs::read_to_string(path).unwrap())
        .unwrap()
        .to_task()
}

/// Builds the heuristic for `task` with exact bisimulation and pruning.
fn build<M: MergeStrategy>(
    task: &Task,
    merge: M,
    label_reduction: bool,
) -> MergeAndShrinkHeuristic {
    MergeAndShrink {
        label_reduction,
        ..MergeAndShrink::new(merge, Bisimulation)
    }
    .build(task)
}

/// Exact bisimulation finds the optimal cost of each translated task, with
/// every merge strategy and with or without label reduction.
#[test]
fn bisimulation_is_perfect_on_translator_output() {
    for (name, optimum) in FIXTURES {
        let task = fixture(name);
        let n = task.num_variables();
        for label_reduction in [false, true] {
            let heuristics = [
                (
                    "linear",
                    build(&task, LinearMerge::in_variable_order(n), label_reduction),
                ),
                ("dfp", build(&task, Dfp, label_reduction)),
                ("scc-dfp", build(&task, SccDfp::new(&task), label_reduction)),
            ];
            for (merge, h) in heuristics {
                assert_eq!(h.value(&task.initial), Some(optimum), "{name}, {merge}");
                println!(
                    "{name}, {merge}, label reduction {label_reduction}: h = {optimum} with {} abstract states",
                    h.num_abstract_states()
                );
            }
        }
    }
}

/// Exact label reduction only combines labels, so bisimulation in a fixed
/// merge order shrinks at least as much with it as without.
#[test]
fn label_reduction_never_enlarges_the_abstraction() {
    for (name, _) in FIXTURES {
        let task = fixture(name);
        let order = || LinearMerge::in_variable_order(task.num_variables());
        let with = build(&task, order(), true).num_abstract_states();
        let without = build(&task, order(), false).num_abstract_states();

        assert!(with <= without, "{name}: {with} > {without}");
    }
}
