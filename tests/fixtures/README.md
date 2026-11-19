# Test fixtures

Fast Downward `output.sas` files copied from [`sas-plus`](https://github.com/sunsided/sas-plus/tree/main/tests/fixtures),
where their format, origin and optimal costs are documented and checked.

- `gripper-costs/`: hand-encoded, written for `sas-plus` (MIT OR Apache-2.0).
- `translated/sussman-anomaly/`: translator output next to its PDDL input.
- `translated/gripper-2/`: translator output next to its PDDL input.
- `translated/gripper-4/`: translator output next to its PDDL input.

## Licences

The PDDL files under `translated/` come from the test data of [sunsided/pddl-rs](https://github.com/sunsided/pddl-rs) (`tests/data/`).
Each `output.sas` is derived from its inputs and falls under the same licences.

- **VHPOP** files come from the `examples/` folder of [VHPOP](https://github.com/hlsyounes/vhpop) by Håkan L. S. Younes, at commit `cf27db2ed46f8bd64cb78fe256cb5cf3ad8174cf`, under the **GNU General Public License, version 2 or later** (`GPL-2.0-or-later`, as VHPOP's sources state); see [`COPYING.vhpop`](https://github.com/sunsided/merge-and-shrink/blob/main/tests/fixtures/translated/COPYING.vhpop). `REUSE.toml` records this per file.

All of this is test input only and is not compiled into the crate. The published package ships only `gripper-costs/`, which the README example reads; `translated/` is left out.
