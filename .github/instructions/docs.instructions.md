---
title: Documentation
applyTo: "**/*.md"
summary: Absolute URLs, doctested README example, consistent badges and MSRV.
---

# Documentation

- `README.md` is rendered on crates.io and docs.rs. Every link to a repo file and every image
  uses an absolute URL (`https://github.com/sunsided/merge-and-shrink/blob/main/...`). Only
  in-page anchors may be relative. `task lint:readme` checks this.
- The README's Rust code blocks run as doctests (`readme_doctests` in `src/lib.rs`), so they
  must compile. Tag other blocks with their language. Use `rust,no_run` or `rust,ignore`
  only with a stated reason.
- Keep the badges, the License section and the MSRV statement consistent with `Cargo.toml`
  and `REUSE.toml`.
- `tests/fixtures/README.md` documents where each fixture comes from and its license. A new
  fixture gets a line there and, if it is not MIT OR Apache-2.0, an entry in `REUSE.toml`.
