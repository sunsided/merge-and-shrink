//! Integration tests for merge-and-shrink, compiled as a single test binary.
//!
//! - [`fast_downward`]: heuristic values compared with Fast Downward (ignored; needs Docker).
//! - [`fdr_gripper`]: a hand-encoded Fast Downward task with action costs.
//! - [`fdr_translated`]: Fast Downward translator output with known optimal costs.
//! - [`merge_strategies`]: which pairs the score-based merge strategies pick.
//! - [`pruning`]: what pruned states evaluate to.
//! - [`random_tasks`]: fixed seeds of random tasks, checked on every state.
//! - [`properties`]: property tests over random tasks and merge orders, checked
//!   against the exact heuristic `h*`.

mod fast_downward;
mod fdr_gripper;
mod fdr_translated;
mod merge_strategies;
mod properties;
mod pruning;
mod random_tasks;
mod strategies;
