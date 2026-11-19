//! Fast Downward translator output with known optimal costs.

use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink};
use sas_plus::fdr::SasFile;

/// Exact bisimulation finds the optimal cost of each translated task.
#[test]
fn bisimulation_is_perfect_on_translator_output() {
    for (name, optimum) in [("sussman-anomaly", 3), ("gripper-2", 3), ("gripper-4", 9)] {
        let path = format!(
            "{}/tests/fixtures/translated/{name}/output.sas",
            env!("CARGO_MANIFEST_DIR")
        );
        let task = SasFile::parse(&std::fs::read_to_string(path).unwrap())
            .unwrap()
            .to_task();
        let h = MergeAndShrink::new(
            LinearMerge::in_variable_order(task.num_variables()),
            Bisimulation,
        )
        .build(&task);
        assert_eq!(h.value(&task.initial), Some(optimum), "{name}");
        println!(
            "{name}: h = {optimum} with {} abstract states",
            h.num_abstract_states()
        );
    }
}
