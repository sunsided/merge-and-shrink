use sas_plus::VarId;

use crate::Factor;

/// Chooses which two active factors to merge next. The product is appended
/// at the end of the factor list (index `factors.len()` at the time of the call).
pub trait MergeStrategy {
    fn next_pair(&mut self, factors: &[Option<Factor>]) -> (usize, usize);
}

/// Merges variables in a fixed order: `((v0 ⊗ v1) ⊗ v2) ⊗ ...`.
#[derive(Clone, Debug)]
pub struct LinearMerge {
    order: Vec<VarId>,
    next: usize,
    composite: Option<usize>,
}

impl LinearMerge {
    pub fn new(order: Vec<VarId>) -> Self {
        Self {
            order,
            next: 0,
            composite: None,
        }
    }

    pub fn in_variable_order(num_vars: usize) -> Self {
        Self::new((0..num_vars).collect())
    }
}

impl MergeStrategy for LinearMerge {
    fn next_pair(&mut self, factors: &[Option<Factor>]) -> (usize, usize) {
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
