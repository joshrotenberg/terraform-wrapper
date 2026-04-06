# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/joshrotenberg/terraform-wrapper/compare/v0.4.0...v0.4.1) - 2026-04-06

### Other

- bump tokio from 1.50.0 to 1.51.0 in the tokio-ecosystem group ([#85](https://github.com/joshrotenberg/terraform-wrapper/pull/85))
- bump tempfile from 3.26.0 to 3.27.0 ([#84](https://github.com/joshrotenberg/terraform-wrapper/pull/84))

## [0.4.0](https://github.com/joshrotenberg/terraform-wrapper/compare/v0.3.0...v0.4.0) - 2026-03-11

### Added

- add state replace-provider subcommand ([#81](https://github.com/joshrotenberg/terraform-wrapper/pull/81))
- add missing flags to Init, Plan, Test, and State commands ([#79](https://github.com/joshrotenberg/terraform-wrapper/pull/79))

### Other

- expand TerraformConfig documentation in README and examples ([#59](https://github.com/joshrotenberg/terraform-wrapper/pull/59))

## [0.3.0](https://github.com/joshrotenberg/terraform-wrapper/compare/v0.2.0...v0.3.0) - 2026-03-04

### Added

- add Display impls for CommandOutput and OutputResult ([#55](https://github.com/joshrotenberg/terraform-wrapper/pull/55))
- add module block support to TerraformConfig ([#54](https://github.com/joshrotenberg/terraform-wrapper/pull/54))
- add providers, test, and refresh commands ([#44](https://github.com/joshrotenberg/terraform-wrapper/pull/44))
- add RawCommand escape hatch for arbitrary subcommands ([#43](https://github.com/joshrotenberg/terraform-wrapper/pull/43))
- add graph, force-unlock, get, and modules commands ([#42](https://github.com/joshrotenberg/terraform-wrapper/pull/42))

### Fixed

- use Error::Json variant for serde parse failures ([#53](https://github.com/joshrotenberg/terraform-wrapper/pull/53))
- add timeout and configurable exit codes to stream_terraform ([#51](https://github.com/joshrotenberg/terraform-wrapper/pull/51))

### Other

- update README, expand command table, and add examples ([#56](https://github.com/joshrotenberg/terraform-wrapper/pull/56))
- consolidate -input=false injection into TerraformCommand trait ([#52](https://github.com/joshrotenberg/terraform-wrapper/pull/52))
- add documentation link to Cargo.toml ([#40](https://github.com/joshrotenberg/terraform-wrapper/pull/40))

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
