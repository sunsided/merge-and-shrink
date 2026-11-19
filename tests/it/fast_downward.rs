//! Compares heuristic values with Fast Downward's merge-and-shrink.
//!
//! Ignored by default because it needs Docker and the Fast Downward image,
//! pinned by digest so that results stay reproducible:
//! `docker pull aibasel/downward@sha256:328732c5cb87e8c34f477a88a4f99977b6ea91ca5c1e859b886b5334c42d80e8`.
//! Run it with `task test:fd`.
//!
//! Each translated fixture is evaluated on states reachable from its initial
//! state: every state becomes the initial state of a copy of the task, and
//! Fast Downward reports its initial heuristic value. With exact bisimulation
//! both heuristics are perfect, so they must agree with `h*`. With a size
//! limit, the two need not agree (not every difference is explained yet), so
//! the values are only reported, and checked to be admissible.

use std::collections::{HashSet, VecDeque};
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use merge_and_shrink::{Bisimulation, GreedyBisimulation, MergeAndShrink, SccDfp, ShrinkStrategy};
use sas_plus::fdr::SasFile;
use sas_plus::{State, Task};

/// The Fast Downward image the comparison runs in.
const IMAGE: &str =
    "aibasel/downward@sha256:328732c5cb87e8c34f477a88a4f99977b6ea91ca5c1e859b886b5334c42d80e8";

/// The fixtures under `tests/fixtures/translated`.
const FIXTURES: [&str; 3] = ["sussman-anomaly", "gripper-2", "gripper-4"];

/// Number of reachable states evaluated per fixture.
const STATES_PER_FIXTURE: usize = 20;

/// One configuration, in both implementations.
struct Config {
    /// Name in the report.
    name: &'static str,
    /// Greedy instead of exact bisimulation.
    greedy: bool,
    /// Size limit; `None` for no limit.
    max_states: Option<usize>,
}

impl Config {
    /// Fast Downward's search argument: SCC-DFP merging (goal relevance, DFP,
    /// total order), exact label reduction before shrinking, bisimulation.
    fn search(&self) -> String {
        let max_states = self
            .max_states
            .map_or_else(|| "infinity".to_owned(), |n| n.to_string());
        format!(
            "astar(merge_and_shrink(\
             shrink_strategy=shrink_bisimulation(greedy={}),\
             merge_strategy=merge_sccs(order_of_sccs=topological,\
             merge_selector=score_based_filtering(\
             scoring_functions=[goal_relevance(),dfp(),total_order()])),\
             label_reduction=exact(before_shrinking=true,before_merging=false),\
             max_states={max_states},threshold_before_merge=1))",
            self.greedy
        )
    }

    /// Our heuristic value of the initial state of `task`, built like
    /// [`Config::search`].
    fn ours(&self, task: &Task) -> Option<u64> {
        if self.greedy {
            self.build(task, GreedyBisimulation)
        } else {
            self.build(task, Bisimulation)
        }
    }

    /// Builds the heuristic for `task` with `shrink` and evaluates its initial state.
    fn build<S: ShrinkStrategy>(&self, task: &Task, shrink: S) -> Option<u64> {
        let h = MergeAndShrink {
            max_states: self.max_states.unwrap_or(usize::MAX),
            ..MergeAndShrink::new(SccDfp::new(task), shrink)
        }
        .build(task);

        h.value(&task.initial)
    }
}

/// One evaluated state: a copy of a fixture with another initial state.
struct Case {
    /// Fixture name and state index, also the file name stem.
    name: String,
    /// The task with that initial state.
    task: Task,
}

/// Exact configurations agree with Fast Downward and `h*` on every evaluated
/// state; bounded ones are admissible in both, and the values are reported.
#[test]
#[ignore = "needs Docker and the aibasel/downward image; run with `task test:fd`"]
fn heuristic_values_match_fast_downward() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fast-downward");
    std::fs::create_dir_all(&dir).unwrap();
    let cases = write_cases(&dir);

    let configs = [
        Config {
            name: "exact",
            greedy: false,
            max_states: None,
        },
        Config {
            name: "bounded-20",
            greedy: false,
            max_states: Some(20),
        },
        Config {
            name: "greedy-20",
            greedy: true,
            max_states: Some(20),
        },
    ];
    for config in &configs {
        let fd = run_fast_downward(&dir, &cases, &config.search());
        let mut agree = 0;
        let mut report = String::new();
        for (case, fd) in cases.iter().zip(fd) {
            let ours = config.ours(&case.task);
            let opt = case.task.perfect_heuristic()(&case.task.initial);
            writeln!(
                report,
                "  {}: h* {opt:?}, ours {ours:?}, fd {fd:?}",
                case.name
            )
            .unwrap();
            agree += usize::from(ours == fd);

            if config.max_states.is_none() {
                assert_eq!(ours, opt, "{}: ours, {}", config.name, case.name);
                assert_eq!(fd, opt, "{}: fd, {}", config.name, case.name);
            } else {
                for (who, h) in [("ours", ours), ("fd", fd)] {
                    assert!(
                        h <= opt || opt.is_none(),
                        "{}: {who} inadmissible, {}",
                        config.name,
                        case.name
                    );
                    assert!(
                        h.is_some() || opt.is_none(),
                        "{}: {who} false dead end, {}",
                        config.name,
                        case.name
                    );
                }
            }
        }

        println!(
            "{}: {agree}/{} states agree\n{report}",
            config.name,
            cases.len()
        );
    }
}

/// Writes one task file per evaluated state into `dir`.
fn write_cases(dir: &Path) -> Vec<Case> {
    let mut cases = Vec::new();
    for fixture in FIXTURES {
        let path = format!(
            "{}/tests/fixtures/translated/{fixture}/output.sas",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut sas = SasFile::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        let states = reachable_states(&sas.to_task(), STATES_PER_FIXTURE);
        for (k, state) in states.into_iter().enumerate() {
            sas.initial = state.0;
            let name = format!("{fixture}-{k}");
            std::fs::write(dir.join(format!("{name}.sas")), sas.to_string()).unwrap();
            cases.push(Case {
                name,
                task: sas.to_task(),
            });
        }
    }

    cases
}

/// Up to `limit` states reachable from the initial state of `task`, in
/// breadth-first order.
fn reachable_states(task: &Task, limit: usize) -> Vec<State> {
    let mut seen = HashSet::from([task.initial.clone()]);
    let mut queue = VecDeque::from([task.initial.clone()]);
    let mut states = Vec::new();
    while let Some(s) = queue.pop_front() {
        if states.len() == limit {
            break;
        }
        for op in task.applicable_operators(&s) {
            let t = task.apply(op, &s);
            if seen.insert(t.clone()) {
                queue.push_back(t);
            }
        }
        states.push(s);
    }

    states
}

/// Runs Fast Downward with `search` on every case in one container and returns
/// its initial heuristic value per case; `None` for a dead end.
fn run_fast_downward(dir: &Path, cases: &[Case], search: &str) -> Vec<Option<u64>> {
    let names: Vec<&str> = cases.iter().map(|c| c.name.as_str()).collect();
    let script = "for n in $NAMES; do echo \"=== $n\"; \
                  python3 /workspace/downward/fast-downward.py /cases/$n.sas --search \"$SEARCH\" 2>&1; \
                  done";
    let output = Command::new("docker")
        .args(["run", "--rm", "-v"])
        .arg(format!("{}:/cases:ro", dir.display()))
        .args(["-e", &format!("NAMES={}", names.join(" "))])
        .args(["-e", &format!("SEARCH={search}")])
        .args(["--entrypoint", "sh", IMAGE, "-c", script])
        .output()
        .expect("docker runs");
    assert!(
        output.status.success(),
        "docker failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let values: Vec<Option<u64>> = stdout.split("=== ").skip(1).map(initial_value).collect();
    assert_eq!(values.len(), cases.len(), "one Fast Downward run per case");

    values
}

/// The initial heuristic value in one Fast Downward log; `None` for a dead end.
///
/// # Panics
///
/// Panics if the log reports neither.
fn initial_value(log: &str) -> Option<u64> {
    const PREFIX: &str = "Initial heuristic value for merge_and_shrink: ";
    if let Some(line) = log
        .lines()
        .find_map(|l| l.split_once(PREFIX).map(|(_, v)| v))
    {
        return match line.trim() {
            "infinity" => None,
            v => Some(
                v.parse()
                    .unwrap_or_else(|_| panic!("bad heuristic value {v:?}")),
            ),
        };
    }

    assert!(
        log.contains("dead end"),
        "no initial heuristic value in:\n{log}"
    );
    None
}
