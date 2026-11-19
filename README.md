# merge-and-shrink

Merge-and-shrink abstraction heuristics for factored transition systems: atomic factors, synchronized products, linear merge strategies, pruning of unreachable and dead states, and exact goal-respecting bisimulation. The resulting heuristic implements `sas_plus::Abstraction`, so it can feed [`cost-partition`](https://github.com/sunsided/cost-partition).

## Usage

Build the heuristic for a task from Fast Downward's translator and evaluate it:

```rust
use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink};
use sas_plus::fdr::SasFile;

# let output_sas = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gripper-costs/output.sas");
// `output_sas` is the path of a translator output file.
let task = SasFile::parse(&std::fs::read_to_string(output_sas)?)?.to_task()?;

let merge = LinearMerge::in_variable_order(task.num_variables());
let h = MergeAndShrink::new(merge, Bisimulation).build(&task);

// Exact bisimulation keeps goal distances: 7 is the optimal plan cost.
assert_eq!(h.value(&task.initial), Some(7));
# Ok::<(), Box<dyn std::error::Error>>(())
```

`None` from `value` means a dead end: no plan exists from that state.

## Tested

Without shrinking, and with exact bisimulation, h equals h* on **every** state of random tasks, for any linear merge order. With pruning, h equals h* on the initial state. These hold as proptest properties and a fuzz target on random tasks, and on translator output.

## Roadmap

Label reduction, greedy and size-bounded bisimulation, shrink-before-merge, DFP / SCC-DFP merging, h-value comparison with Fast Downward.

## References

Helmert, Haslum, Hoffmann & Nissim (JACM 2014); Sievers & Helmert (JAIR 2021).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSES/Apache-2.0.txt](https://github.com/sunsided/merge-and-shrink/blob/main/LICENSES/Apache-2.0.txt))
- MIT license ([LICENSES/MIT.txt](https://github.com/sunsided/merge-and-shrink/blob/main/LICENSES/MIT.txt))

at your option. Test fixtures derived from VHPOP are GPL-2.0-or-later and are not part of the published crate; see [`REUSE.toml`](https://github.com/sunsided/merge-and-shrink/blob/main/REUSE.toml).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
