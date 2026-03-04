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
use terraform_wrapper::prelude::*;

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

Note: You must import the `TerraformCommand` trait to call `.execute()`. The `prelude` module re-exports everything you need.

## Commands

| Command | Description |
|---------|-------------|
| `InitCommand` | Prepare working directory, download providers |
| `ValidateCommand` | Check configuration validity |
| `PlanCommand` | Preview infrastructure changes |
| `ApplyCommand` | Create or update infrastructure |
| `DestroyCommand` | Destroy infrastructure |
| `OutputCommand` | Read output values |
| `ShowCommand` | Inspect current state or saved plan |
| `FmtCommand` | Format configuration files |
| `WorkspaceCommand` | Manage workspaces (list, new, select, delete) |
| `StateCommand` | Advanced state management (list, show, mv, rm) |
| `ImportCommand` | Import existing infrastructure into state |
| `VersionCommand` | Get Terraform version info |

## Streaming Output

Long-running commands like `apply` produce streaming JSON events. Use `stream_terraform` to process them in real-time:

```rust,no_run
use terraform_wrapper::prelude::*;
use terraform_wrapper::streaming::{stream_terraform, JsonLogLine};

# async fn example() -> terraform_wrapper::error::Result<()> {
# let tf = Terraform::builder().build()?;
let result = stream_terraform(
    &tf,
    ApplyCommand::new().auto_approve().json(),
    &[0],
    |line: JsonLogLine| {
        println!("[{}] {}", line.log_type, line.message);
    },
).await?;
# Ok(())
# }
```

See the [`streaming_apply` example](examples/streaming_apply.rs) for a complete working example.

## Config Builder

Define Terraform configs entirely in Rust -- no `.tf` files needed. Enable the `config` feature:

```toml
[dependencies]
terraform-wrapper = { version = "0.1", features = ["config"] }
```

```rust,no_run
use terraform_wrapper::config::TerraformConfig;
use serde_json::json;

# fn example() -> std::io::Result<()> {
let config = TerraformConfig::new()
    .required_provider("aws", "hashicorp/aws", "~> 5.0")
    .provider("aws", json!({ "region": "us-west-2" }))
    .resource("aws_instance", "web", json!({
        "ami": "ami-0c55b159",
        "instance_type": "t3.micro"
    }))
    .output("id", json!({ "value": "${aws_instance.web.id}" }));

let dir = config.write_to_tempdir()?;
// Terraform::builder().working_dir(dir.path()).build()?;
# Ok(())
# }
```

See the [`config_builder` example](examples/config_builder.rs) for a complete working example.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `json` | Yes | Typed JSON output parsing via `serde` / `serde_json` |
| `config` | No | `TerraformConfig` builder for `.tf.json` generation |

Disable defaults for raw command output only:

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
