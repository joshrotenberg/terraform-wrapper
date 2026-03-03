# terraform-wrapper

[![Crates.io](https://img.shields.io/crates/v/terraform-wrapper.svg)](https://crates.io/crates/terraform-wrapper)
[![Documentation](https://docs.rs/terraform-wrapper/badge.svg)](https://docs.rs/terraform-wrapper)
[![CI](https://github.com/joshrotenberg/terraform-wrapper/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/terraform-wrapper/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/terraform-wrapper.svg)](https://github.com/joshrotenberg/terraform-wrapper#license)

A type-safe Terraform CLI wrapper for Rust.

## Installation

```toml
[dependencies]
terraform-wrapper = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Minimum supported Rust version: 1.85.0

## Quick Start

```rust,no_run
use terraform_wrapper::{Terraform, TerraformCommand};
use terraform_wrapper::commands::init::InitCommand;
use terraform_wrapper::commands::apply::ApplyCommand;
use terraform_wrapper::commands::output::{OutputCommand, OutputResult};
use terraform_wrapper::commands::destroy::DestroyCommand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tf = Terraform::builder()
        .working_dir("./infra")
        .build()?;

    // Initialize providers
    InitCommand::new().execute(&tf).await?;

    // Apply changes
    ApplyCommand::new()
        .auto_approve()
        .var("region", "us-west-2")
        .execute(&tf)
        .await?;

    // Read outputs
    let result = OutputCommand::new()
        .name("endpoint")
        .raw()
        .execute(&tf)
        .await?;

    if let OutputResult::Raw(value) = result {
        println!("Endpoint: {value}");
    }

    // Tear down
    DestroyCommand::new()
        .auto_approve()
        .execute(&tf)
        .await?;

    Ok(())
}
```

Note: You must import the `TerraformCommand` trait to call `.execute()`.

## Commands

| Command | Description |
|---------|-------------|
| `InitCommand` | Prepare working directory, download providers |
| `ValidateCommand` | Check configuration validity |
| `PlanCommand` | Preview infrastructure changes |
| `ApplyCommand` | Create or update infrastructure |
| `DestroyCommand` | Destroy infrastructure |
| `OutputCommand` | Read output values |
| `VersionCommand` | Get Terraform version info |

## Features

JSON output parsing is enabled by default. Disable it if you only need raw command output:

```toml
[dependencies]
terraform-wrapper = { version = "0.1", default-features = false }
```

## Why terraform-wrapper?

| | terraform-wrapper | terrars | terraform-rs |
|---|---|---|---|
| Approach | CLI wrapper | CDK-style codegen | Minimal CLI wrapper |
| Async | Yes (tokio) | No | No |
| JSON output | Typed structs | N/A | No |
| Maintained | Active | Active | Unmaintained (2021) |
| Use case | Orchestration tools | Generate .tf in Rust | Basic CLI calls |

Use `terraform-wrapper` when you need to programmatically drive Terraform lifecycles (provision, extract outputs, tear down) from Rust with type-safe, async APIs.

## Documentation

Full API reference is available on [docs.rs](https://docs.rs/terraform-wrapper).

See the [`examples/`](examples/) directory for working examples.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
