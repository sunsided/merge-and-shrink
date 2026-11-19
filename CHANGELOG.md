# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/sunsided/merge-and-shrink/compare/v0.2.1...v0.3.0) - 2026-11-19

### Added

- Allow the unparseable spelling in typos
- Add exact label reduction and DFP / SCC-DFP merging

### Changed

- Pin the Fast Downward image and reject unparseable values
- Find a factor's component by its first variable in SCC-DFP
- Guard bisimulation against overflow and a zero size limit
- Set the release version to 0.3.0
- Stop bounded bisimulation at the first split over the limit
- Document the new merge, shrink and label reduction options
- Compare heuristic values with Fast Downward
- Shrink before merging, within a size limit
- Test pruning on a factor that keeps live states
- Test that pruned goal states evaluate to None
- State what a shrink strategy must keep for pruning to hold
- Count bisimulation blocks while numbering them
- Check the pruned heuristic on every reachable state
- Prune each product before shrinking it
- Document that pruned states evaluate to None

### Fixed

- Correct the shrink contract on None entries

## [0.2.1](https://github.com/sunsided/merge-and-shrink/compare/v0.2.0...v0.2.1) - 2026-11-19

### Changed

- Make the library no_std
- Explain what the crate is for, how it works and who needs it

### Fixed

- Correct the README's bisimulation and perfection claims

## [0.2.0](https://github.com/sunsided/merge-and-shrink/compare/v0.1.0...v0.2.0) - 2026-11-19

### Added

- Add the semver job to CI

### Changed

- Pin the nightly fuzz job to the same rust-toolchain commit
- Upgrade sas-plus to 0.2, criterion to 0.8 and proptest to 1.12

### Fixed

- Fix merge tree lookups on tasks with derived variables
