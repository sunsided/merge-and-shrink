use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink};
use sas_plus::fdr::SasFile;

#[test]
fn bisimulation_is_perfect_on_gripper_with_action_costs() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/gripper-costs/output.sas"
    );
    let task = SasFile::parse(&std::fs::read_to_string(path).unwrap())
        .unwrap()
        .to_task()
        .unwrap();
    let h = MergeAndShrink::new(
        LinearMerge::in_variable_order(task.num_variables()),
        Bisimulation,
    )
    .build(&task);
    assert_eq!(h.value(&task.initial), Some(7));
}
