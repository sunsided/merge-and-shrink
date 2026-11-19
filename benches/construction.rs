//! Benchmarks for building and evaluating merge-and-shrink heuristics.
//!
//! The input is the translated `gripper-4` fixture, merged in variable order.

use std::hint::black_box;

use criterion::Criterion;
use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink, NoShrink};
use sas_plus::Task;
use sas_plus::fdr::SasFile;

/// Reads the translated `gripper-4` fixture.
fn gripper_4() -> Task {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/translated/gripper-4/output.sas"
    );
    let text = std::fs::read_to_string(path).expect("fixture is readable");

    SasFile::parse(&text).expect("fixture parses").to_task()
}

/// Times building the heuristic with and without exact bisimulation, and
/// evaluating the bisimulation heuristic on the initial state.
fn bench_construction(criterion: &mut Criterion) {
    let task = gripper_4();
    let order = || LinearMerge::in_variable_order(task.num_variables());

    criterion.bench_function("no_shrink/build", |b| {
        b.iter(|| MergeAndShrink::new(order(), NoShrink).build(black_box(&task)));
    });
    criterion.bench_function("bisimulation/build", |b| {
        b.iter(|| MergeAndShrink::new(order(), Bisimulation).build(black_box(&task)));
    });

    let h = MergeAndShrink::new(order(), Bisimulation).build(&task);
    criterion.bench_function("bisimulation/value", |b| {
        b.iter(|| h.value(black_box(&task.initial)));
    });
}

/// Runs every benchmark in this file with criterion's command-line flags.
fn main() {
    let mut criterion = Criterion::default().configure_from_args();

    bench_construction(&mut criterion);
    criterion.final_summary();
}
