---
title: Crate and releases
applyTo: "Cargo.toml,release-plz.toml,.github/workflows/release-plz.yml,.github/workflows/publish-crates-io.yml,CHANGELOG.md"
summary: Crate metadata, release-plz versions and changelog, deny policy, trusted publishing.
---

# Crate and releases

- Versions are SemVer 2.0 as Cargo reads it; in `0.x` the minor number is the breaking
  position. Pre-releases (`-rc.1`) are fine, build metadata (`+...`) is not.
- release-plz release PRs manage the version and `CHANGELOG.md`. Feature PRs do not bump the
  version or edit generated changelog entries.
- `Cargo.toml` keeps `description`, `license`, `repository`, `readme`, `keywords` (at most 5)
  and `categories` (valid crates.io slugs), and declares `rust-version`.
- The library depends on `sas-plus` with `default-features = false`; features the tests need
  go on the dev-dependency only.
- `fuzz/Cargo.toml` uses the same `sas-plus` version requirement as the crate, so the task
  types unify. Bump both together.
- New dependencies must pass `cargo deny check`: allowed licenses, no advisories, no git or
  unknown-registry sources.
- `rust-version` is a signal, not a contract. Raising it needs a minor release and a changelog
  note, never a patch release.
- release-plz runs in git-only mode: it tags `vX.Y.Z` and creates the GitHub Release.
  `publish-crates-io.yml` publishes from the tag in the `crates-io` environment via trusted
  publishing (`id-token: write`, no `CARGO_REGISTRY_TOKEN`). Keep it that way.
- Changelog groups and minor bumps come from imperative subject verbs (`release-plz.toml`). A
  subject that hides a feature behind "Update ..." lands under Changed. Flag it.
