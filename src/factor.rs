//! Factors of a factored transition system and the merge trees that map
//! concrete states into them.

use sas_plus::{Abstraction, LabeledTransitionSystem, Projection, State, Task, Transition, VarId};

/// Maps concrete states to abstract states of a factor ("cascading tables").
#[derive(Clone, Debug)]
///
/// Each node owns a table from the (combined) states of its children to abstract
/// states. Shrinking and pruning rewrite only the root table; `None` entries are
/// states that were pruned.
pub enum MergeTree {
    /// An atomic factor: one task variable.
    Leaf {
        /// The task variable this leaf reads from the state.
        var: VarId,
        /// Abstract state per value of `var`, indexed by value; `None` if pruned.
        table: Vec<Option<usize>>,
    },
    /// The synchronized product of two factors.
    Node {
        /// The tree of the left factor; its abstract state is the major index.
        left: Box<MergeTree>,
        /// The tree of the right factor; its abstract state is the minor index.
        right: Box<MergeTree>,
        /// Number of abstract states of the right factor at the time of the product.
        right_size: usize,
        /// Abstract state per product state `left * right_size + right`; `None` if pruned.
        table: Vec<Option<usize>>,
    },
}

impl MergeTree {
    /// The abstract state of a concrete `state`, or `None` if it was pruned.
    ///
    /// # Panics
    ///
    /// Panics if `state` has fewer variables or larger values than the task the
    /// tree was built from.
    #[must_use]
    pub fn lookup(&self, state: &State) -> Option<usize> {
        match self {
            Self::Leaf { var, table } => table[state.0[*var] as usize],
            Self::Node {
                left,
                right,
                right_size,
                table,
            } => {
                let l = left.lookup(state)?;
                let r = right.lookup(state)?;

                table[l * right_size + r]
            }
        }
    }

    /// Maps every root table entry through `map` (old abstract state to new).
    fn compose(&mut self, map: &[Option<usize>]) {
        let (Self::Leaf { table, .. } | Self::Node { table, .. }) = self;
        for entry in table {
            *entry = entry.and_then(|s| map[s]);
        }
    }
}

/// One factor of a factored transition system.
#[derive(Clone, Debug)]
pub struct Factor {
    /// The abstract transition system; labels are the task's operator ids.
    pub lts: LabeledTransitionSystem,
    /// Maps concrete states to states of `lts`.
    pub tree: MergeTree,
}

impl Factor {
    /// The atomic transition system of one variable.
    ///
    /// A derived variable has a single abstract state that all of its values
    /// map to: projections drop derived variables and treat conditions on them
    /// as satisfiable.
    #[must_use]
    pub fn atomic(task: &Task, var: VarId) -> Self {
        let p = Projection::new(task, vec![var]);
        let table = if p.pattern().is_empty() {
            vec![Some(0); task.variables[var].domain_size as usize]
        } else {
            (0..p.lts().num_states).map(Some).collect()
        };

        Self {
            lts: p.lts().clone(),
            tree: MergeTree::Leaf { var, table },
        }
    }

    /// Synchronized product: transitions with the same label happen together.
    ///
    /// Product state `(x, y)` is numbered `x * b.lts.num_states + y`.
    ///
    /// # Panics
    ///
    /// Panics if the factors have different numbers of labels.
    #[must_use]
    pub fn product(a: Factor, b: Factor) -> Factor {
        assert_eq!(a.lts.num_labels, b.lts.num_labels);
        let nb = b.lts.num_states;
        let transitions = synchronized_transitions(&a.lts, &b.lts);
        let num_states = a.lts.num_states * nb;
        let goal = (0..num_states)
            .map(|s| a.lts.goal[s / nb] && b.lts.goal[s % nb])
            .collect();
        let initial = a.lts.initial.zip(b.lts.initial).map(|(x, y)| x * nb + y);

        Factor {
            lts: LabeledTransitionSystem {
                num_states,
                num_labels: a.lts.num_labels,
                transitions,
                goal,
                initial,
            },
            tree: MergeTree::Node {
                left: Box::new(a.tree),
                right: Box::new(b.tree),
                right_size: nb,
                table: (0..num_states).map(Some).collect(),
            },
        }
    }

    /// Replaces the factor by its quotient under `map` (old state to new
    /// state in `0..num_new`, or `None` to prune).
    ///
    /// An abstract state is a goal if any state mapped to it is one. Duplicate
    /// transitions are merged.
    ///
    /// # Panics
    ///
    /// Panics if `map` does not have one entry per state, or maps a state to
    /// `num_new` or above.
    pub fn apply_abstraction(&mut self, map: &[Option<usize>], num_new: usize) {
        assert_eq!(map.len(), self.lts.num_states);
        let mut goal = vec![false; num_new];
        for (s, m) in map.iter().enumerate() {
            if let Some(m) = *m {
                goal[m] |= self.lts.goal[s];
            }
        }

        let mut transitions: Vec<Transition> = self
            .lts
            .transitions
            .iter()
            .filter_map(|t| {
                Some(Transition {
                    src: map[t.src]?,
                    label: t.label,
                    dst: map[t.dst]?,
                })
            })
            .collect();
        transitions.sort_unstable();
        transitions.dedup();

        self.lts = LabeledTransitionSystem {
            num_states: num_new,
            num_labels: self.lts.num_labels,
            transitions,
            goal,
            initial: self.lts.initial.and_then(|s| map[s]),
        };
        self.tree.compose(map);
    }

    /// Prunes states unreachable from the initial state or unable to reach a goal.
    ///
    /// `costs` holds one cost per label (operator id).
    pub fn prune(&mut self, costs: &[u64]) {
        let g = self.lts.goal_distances(costs);
        let i = self.lts.init_distances(costs);
        let mut n = 0;
        let map: Vec<Option<usize>> = (0..self.lts.num_states)
            .map(|s| {
                (g[s].is_some() && i[s].is_some()).then(|| {
                    n += 1;
                    n - 1
                })
            })
            .collect();

        if n < self.lts.num_states {
            self.apply_abstraction(&map, n);
        }
    }
}

/// Transitions of the synchronized product of `a` and `b`: one per pair of
/// transitions with the same label.
///
/// Product state `(x, y)` is numbered `x * b.num_states + y`.
fn synchronized_transitions(
    a: &LabeledTransitionSystem,
    b: &LabeledTransitionSystem,
) -> Vec<Transition> {
    let nb = b.num_states;
    let mut by_label: Vec<Vec<(usize, usize)>> = vec![Vec::new(); b.num_labels];
    for t in &b.transitions {
        by_label[t.label].push((t.src, t.dst));
    }

    let mut transitions = Vec::new();
    for t in &a.transitions {
        for &(bs, bd) in &by_label[t.label] {
            transitions.push(Transition {
                src: t.src * nb + bs,
                label: t.label,
                dst: t.dst * nb + bd,
            });
        }
    }

    transitions
}
