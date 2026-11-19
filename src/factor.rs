use sas_plus::{Abstraction, LabeledTransitionSystem, Projection, State, Task, Transition, VarId};

/// Maps concrete states to abstract states of a factor ("cascading tables").
#[derive(Clone, Debug)]
pub enum MergeTree {
    Leaf {
        var: VarId,
        table: Vec<Option<usize>>,
    },
    Node {
        left: Box<MergeTree>,
        right: Box<MergeTree>,
        right_size: usize,
        table: Vec<Option<usize>>,
    },
}

impl MergeTree {
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
    pub lts: LabeledTransitionSystem,
    pub tree: MergeTree,
}

impl Factor {
    /// The atomic transition system of one variable.
    pub fn atomic(task: &Task, var: VarId) -> Self {
        let p = Projection::new(task, vec![var]);
        let n = p.lts().num_states;
        Self {
            lts: p.lts().clone(),
            tree: MergeTree::Leaf {
                var,
                table: (0..n).map(Some).collect(),
            },
        }
    }

    /// Synchronized product: transitions with the same label happen together.
    pub fn product(a: Factor, b: Factor) -> Factor {
        assert_eq!(a.lts.num_labels, b.lts.num_labels);
        let nb = b.lts.num_states;
        let mut by_label: Vec<Vec<(usize, usize)>> = vec![Vec::new(); b.lts.num_labels];
        for t in &b.lts.transitions {
            by_label[t.label].push((t.src, t.dst));
        }
        let mut transitions = Vec::new();
        for t in &a.lts.transitions {
            for &(bs, bd) in &by_label[t.label] {
                transitions.push(Transition {
                    src: t.src * nb + bs,
                    label: t.label,
                    dst: t.dst * nb + bd,
                });
            }
        }
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

    /// Replaces the factor by its quotient under `map` (old state -> new
    /// state in `0..num_new`, or `None` to prune).
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
