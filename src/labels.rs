//! Exact label reduction (Sievers, Wehrle & Helmert 2014): labels that no
//! factor but one can tell apart are combined into one label in every factor.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use sas_plus::{LabeledTransitionSystem, Transition};

use crate::Factor;
use crate::shrink::assign_ids;

/// Which label each operator has been reduced to.
///
/// A reduced label is represented by one of its operators, so it keeps that
/// operator's id and cost: only labels of equal cost are combined.
pub(crate) struct Labels {
    /// The representative of each operator's label; `rep[l] == l` for representatives.
    rep: Vec<usize>,
}

impl Labels {
    /// No reductions: every operator is its own label.
    pub(crate) fn new(num_labels: usize) -> Self {
        Self {
            rep: (0..num_labels).collect(),
        }
    }

    /// Combines labels of equal cost that have the same transitions in every
    /// active factor other than `except`, and relabels every factor.
    ///
    /// This is exact: products and goal distances are as if the labels had
    /// been kept apart (combinability, Sievers et al. 2014).
    pub(crate) fn reduce(&mut self, factors: &mut [Option<Factor>], except: usize, costs: &[u64]) {
        let reps: Vec<usize> = (0..self.rep.len()).filter(|&l| self.rep[l] == l).collect();
        let mut keys: Vec<(u64, Vec<usize>)> =
            reps.iter().map(|&l| (costs[l], Vec::new())).collect();
        let others = factors
            .iter()
            .enumerate()
            .filter(|&(k, _)| k != except)
            .filter_map(|(_, f)| f.as_ref());
        for factor in others {
            let classes = local_classes(&factor.lts);
            for (key, &l) in keys.iter_mut().zip(&reps) {
                key.1.push(classes[l]);
            }
        }

        let mut first: BTreeMap<&(u64, Vec<usize>), usize> = BTreeMap::new();
        let mut new_rep = self.rep.clone();
        let mut changed = false;
        for (key, &l) in keys.iter().zip(&reps) {
            let r = *first.entry(key).or_insert(l);
            changed |= r != l;
            new_rep[l] = r;
        }
        if !changed {
            return;
        }

        for r in &mut self.rep {
            *r = new_rep[*r];
        }
        for factor in factors.iter_mut().flatten() {
            relabel(&mut factor.lts, &self.rep);
        }
    }

    /// Gives every operator the transitions of its label, so that `lts`
    /// is labeled with operator ids again.
    pub(crate) fn expand(&self, lts: &mut LabeledTransitionSystem) {
        let mut members = vec![Vec::new(); self.rep.len()];
        for (l, &r) in self.rep.iter().enumerate() {
            members[r].push(l);
        }

        lts.transitions = lts
            .transitions
            .iter()
            .flat_map(|t| {
                members[t.label].iter().map(|&label| Transition {
                    src: t.src,
                    label,
                    dst: t.dst,
                })
            })
            .collect();
        lts.transitions.sort_unstable();
    }
}

/// Local equivalence classes of the labels of `lts`: labels with the same
/// transitions get the same class id.
fn local_classes(lts: &LabeledTransitionSystem) -> Vec<usize> {
    let mut by_label: Vec<Vec<(usize, usize)>> = vec![Vec::new(); lts.num_labels];
    for t in &lts.transitions {
        by_label[t.label].push((t.src, t.dst));
    }
    for edges in &mut by_label {
        edges.sort_unstable();
        edges.dedup();
    }

    assign_ids(by_label.into_iter()).0
}

/// Replaces each transition label by its representative and drops duplicates.
fn relabel(lts: &mut LabeledTransitionSystem, rep: &[usize]) {
    for t in &mut lts.transitions {
        t.label = rep[t.label];
    }
    lts.transitions.sort_unstable();
    lts.transitions.dedup();
}
