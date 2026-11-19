# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/sunsided/merge-and-shrink/compare/v0.1.0...v1.0.0) - 2026-11-19

### Added

- Add the semver job to CI

### Changed

- Pin the nightly fuzz job to the same rust-toolchain commit
- Upgrade sas-plus to 0.2, criterion to 0.8 and proptest to 1.12

### Fixed

- Fix merge tree lookups on tasks with derived variables
