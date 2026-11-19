# merge-and-shrink

Merge-and-shrink abstraction heuristics for factored transition systems: atomic factors, synchronized products, linear merge strategies, pruning of unreachable and dead states, and exact goal-respecting bisimulation. The resulting heuristic implements `sas_plus::Abstraction`, so it can feed [`cost-partition`](https://github.com/sunsided/cost-partition).

**Tested:** without shrinking, and with exact bisimulation, h equals h* on **every** state of random tasks.

**Roadmap:** label reduction, greedy and size-bounded bisimulation, shrink-before-merge, DFP / SCC-DFP merging, h-value comparison with Fast Downward.

References: Helmert, Haslum, Hoffmann & Nissim (JACM 2014); Sievers & Helmert (JAIR 2021).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSES/Apache-2.0.txt](https://github.com/sunsided/merge-and-shrink/blob/main/LICENSES/Apache-2.0.txt))
- MIT license ([LICENSES/MIT.txt](https://github.com/sunsided/merge-and-shrink/blob/main/LICENSES/MIT.txt))

at your option. Test fixtures derived from VHPOP are GPL-2.0-or-later and are not part of the published crate; see [`REUSE.toml`](https://github.com/sunsided/merge-and-shrink/blob/main/REUSE.toml).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
