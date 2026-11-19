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

- **VHPOP** files come from the `examples/` folder of [VHPOP](https://github.com/hlsyounes/vhpop) by Håkan L. S. Younes, at commit `cf27db2ed46f8bd64cb78fe256cb5cf3ad8174cf`, under the **GNU General Public License, version 2**; see [`COPYING.vhpop`](translated/COPYING.vhpop).

All of this is test input only. It is not compiled into the crate and is excluded from its published package.
