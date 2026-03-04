# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/joshrotenberg/terraform-wrapper/compare/v0.1.1...v0.2.0) - 2026-03-03

### Added

- add TerraformConfig builder for .tf.json generation ([#39](https://github.com/joshrotenberg/terraform-wrapper/pull/39))
- add streaming JSON output support ([#37](https://github.com/joshrotenberg/terraform-wrapper/pull/37))
- add per-command working directory override ([#36](https://github.com/joshrotenberg/terraform-wrapper/pull/36))
- add command timeout support ([#34](https://github.com/joshrotenberg/terraform-wrapper/pull/34))
- add re-exports and prelude module for ergonomic imports ([#33](https://github.com/joshrotenberg/terraform-wrapper/pull/33))
- add GCP Compute Engine example ([#30](https://github.com/joshrotenberg/terraform-wrapper/pull/30))

### Other

- enrich rustdoc with comprehensive front page documentation ([#35](https://github.com/joshrotenberg/terraform-wrapper/pull/35))

## [0.1.1](https://github.com/joshrotenberg/terraform-wrapper/compare/v0.1.0...v0.1.1) - 2026-03-03

### Added

- add FmtCommand, WorkspaceCommand, StateCommand, ImportCommand ([#19](https://github.com/joshrotenberg/terraform-wrapper/pull/19))
- add ShowCommand with state/plan JSON types ([#15](https://github.com/joshrotenberg/terraform-wrapper/pull/15))

### Other

- bump hashicorp/setup-terraform from 3 to 4 ([#12](https://github.com/joshrotenberg/terraform-wrapper/pull/12))
- bump actions/checkout from 4 to 6 ([#13](https://github.com/joshrotenberg/terraform-wrapper/pull/13))
- release v0.1.0 ([#10](https://github.com/joshrotenberg/terraform-wrapper/pull/10))

## [0.1.0](https://github.com/joshrotenberg/terraform-wrapper/releases/tag/v0.1.0) - 2026-03-03

### Added

- add ValidateCommand, README, move design doc to issues ([#9](https://github.com/joshrotenberg/terraform-wrapper/pull/9))
- initial implementation of terraform-wrapper

### Fixed

- update license copyright year to 2026
