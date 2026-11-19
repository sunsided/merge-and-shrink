//! Merge-and-shrink abstractions (Helmert, Haslum, Hoffmann & Nissim 2014;
//! Sievers & Helmert 2021).
//!
//! Start from one atomic transition system per variable, then repeatedly
//! *merge* two factors into their synchronized product and *shrink* the result
//! by a state equivalence. Goal distances in the final factor are an
//! admissible (and, with exact bisimulation, perfect) heuristic.

mod factor;
mod merge;
mod shrink;

pub use factor::{Factor, MergeTree};
pub use merge::{LinearMerge, MergeStrategy};
pub use shrink::{Bisimulation, NoShrink, ShrinkStrategy};
