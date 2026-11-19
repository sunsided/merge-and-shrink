---
title: CI, Taskfile and hooks
applyTo: ".github/workflows/**,Taskfile.dist.yaml,**/Taskfile.dist.yaml,.pre-commit-config.yaml,scripts/**"
summary: Keep task ci, ci.yml and hooks in sync; pinned, least-privilege workflows.
---

# CI, Taskfile and hooks stay in sync

- `task ci` in `Taskfile.dist.yaml` mirrors the jobs in `.github/workflows/ci.yml`: `branch`
  (`lint:branch`), `fmt` (`fmt:check`), `clippy`, `style` (`lint:separators`, `lint:readme`,
  `lint:typos`), `doc`, `test`, `deny`, `msrv` and `fuzz` (`fuzz:build`, `fuzz:lint`).
  `coverage` is CI-only by design. A change that
  adds, removes or renames a job, or changes its flags, changes both files.
- `.github/workflows/actions-lint.yml` runs actionlint and zizmor, like `task lint:actions`.
- Local hooks in `.pre-commit-config.yaml` call Taskfile tasks and do not duplicate commands.
  The whitespace fixers exclude `LICENSES/` and `tests/fixtures/`, which are verbatim copies.
- Every `uses:` is pinned to a full 40-character commit SHA with a version comment. Reject tag
  or branch refs.
- Workflows set `permissions: {}` at the top, and each job the least privilege it needs.
  Every `actions/checkout` sets `persist-credentials: false`.
- No `pull_request_target`. No `${{ github.event.* }}` or other untrusted expressions directly
  in `run:`; pass them through `env:`.
- Do not rename `publish-crates-io.yml` or its `crates-io` environment: crates.io trusted
  publishing is bound to both.
- `scripts/check-branch.sh` is the single definition of allowed branch names (GitHub Flow).
- New jobs that should gate merges must also be added to the ruleset's required checks. Say so
  in the PR.
