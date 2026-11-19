//! Which pairs the score-based merge strategies pick.

use merge_and_shrink::{Factor, MergeStrategy, SccDfp};
use sas_plus::fdr::SasFile;

/// In translated `gripper-4`, the causal graph has the components `{0}` and
/// `{1, ..., 6}` (as Fast Downward reports), so SCC-DFP merges the six
/// variables of the larger one into one factor before it touches variable 0.
#[test]
fn scc_dfp_merges_within_the_component_first() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/translated/gripper-4/output.sas"
    );
    let task = SasFile::parse(&std::fs::read_to_string(path).unwrap())
        .unwrap()
        .to_task();
    let costs = task.costs();
    let mut factors: Vec<Option<Factor>> = (0..task.num_variables())
        .map(|v| Some(Factor::atomic(&task, v)))
        .collect();

    let mut merge = SccDfp::new(&task);
    for _ in 0..5 {
        let (i, j) = merge.next_pair(&factors, &costs);
        assert!(i != 0 && j != 0, "merged variable 0 early: ({i}, {j})");
        let a = factors[i].take().unwrap();
        let b = factors[j].take().unwrap();
        factors.push(Some(Factor::product(a, b)));
    }

    assert_eq!(merge.next_pair(&factors, &costs), (0, factors.len() - 1));
}
