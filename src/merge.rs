//! Merge strategies: which two factors to combine next.

mod dfp;

pub use dfp::{Dfp, SccDfp};

use alloc::vec::Vec;

use sas_plus::VarId;

use crate::Factor;

/// Chooses which two active factors to merge next. The product is appended
/// at the end of the factor list (index `factors.len()` at the time of the call).
pub trait MergeStrategy {
    /// The indices of the next two factors to merge, in that order.
    ///
    /// `factors` holds every factor created so far; merged ones are `None`.
    /// At least two are active. Both indices must point at active (`Some`)
    /// factors and differ. `costs` holds one cost per label; with label
    /// reduction, factors only use representative labels, which keep their
    /// operator's cost.
    fn next_pair(&mut self, factors: &[Option<Factor>], costs: &[u64]) -> (usize, usize);
}

/// Merges variables in a fixed order: `((v0 ⊗ v1) ⊗ v2) ⊗ ...`.
#[derive(Clone, Debug)]
pub struct LinearMerge {
    /// The variables in merge order; see [`LinearMerge::new`].
    order: Vec<VarId>,
    /// Number of entries of `order` merged so far.
    next: usize,
    /// Index of the composite factor built so far; `None` before the first merge.
    composite: Option<usize>,
}

impl LinearMerge {
    /// Merges the atomic factors of `order` from first to last.
    ///
    /// `order` should list every variable of the task once. Building panics if
    /// it has fewer entries than the task has variables, or repeats a variable
    /// within those; entries beyond the task's variables are never read.
    #[must_use]
    pub fn new(order: Vec<VarId>) -> Self {
        Self {
            order,
            next: 0,
            composite: None,
        }
    }

    /// Merges variables `0..num_vars` in index order.
    #[must_use]
    pub fn in_variable_order(num_vars: usize) -> Self {
        Self::new((0..num_vars).collect())
    }
}

impl MergeStrategy for LinearMerge {
    fn next_pair(&mut self, factors: &[Option<Factor>], _: &[u64]) -> (usize, usize) {
        let pair = match self.composite {
            None => {
                self.next = 2;
                (self.order[0], self.order[1])
            }
            Some(c) => {
                self.next += 1;
                (c, self.order[self.next - 1])
            }
        };
        self.composite = Some(factors.len());

        pair
    }
}
