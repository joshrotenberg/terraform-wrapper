# terraform-wrapper Design Document

## Overview

`terraform-wrapper` is a Rust crate providing a type-safe, async, builder-pattern CLI wrapper around the Terraform binary. It follows the same architectural patterns as `docker-wrapper` — subprocess-based execution, builder per command, a shared trait for execution, and structured output parsing.

A typical consumer would be an infrastructure orchestration tool that needs to programmatically drive Terraform lifecycles — provisioning cloud resources, extracting outputs like connection strings or IPs, and tearing down environments cleanly.

## Motivation

The Rust ecosystem has no viable Terraform CLI wrapper:

- **`terraform` crate** (PhilippeChepy/terraform-rs): Minimal, unmaintained since 2021.
- **`terrars`**: CDK-like approach for generating Terraform stacks in Rust — not a CLI wrapper.
- **`tfschema-bindgen`**: Schema-to-Serde type generation — complementary but different purpose.

The gap is identical to where Docker was before `docker-wrapper`: lots of API-level clients, no good CLI wrapper with ergonomic Rust patterns.

## Architectural Parallels to docker-wrapper

| Concept | docker-wrapper | terraform-wrapper |
|---|---|---|
| Execution model | Shell out to `docker` CLI | Shell out to `terraform` CLI |
| Command pattern | Builder per command (`RunCommand`, `BuildCommand`) | Builder per command (`InitCommand`, `ApplyCommand`) |
| Execution trait | `DockerCommand` trait with `.execute()` | `TerraformCommand` trait with `.execute()` |
| Async runtime | tokio | tokio |
| Output handling | Parsed stdout/stderr | Parsed stdout/stderr + `-json` structured output |
| Error handling | `DockerError` enum | `TerraformError` enum |
| Client struct | `Docker` (optional binary path) | `Terraform` (optional binary path) |

## Key Difference: Structured JSON Output

Terraform has a significant advantage over Docker: most commands support `-json` flags producing machine-readable output. This means we can provide strongly-typed Rust structs for plan output, state, resource changes, and diagnostics — not just raw strings. This should be a first-class feature of the crate.

## Key Difference: Working Directory

Terraform is inherently directory-scoped. Configuration files, state, and the `.terraform` directory all live relative to a working directory. Every command needs to know which directory to operate in.

This is actually *easier* programmatically than interactively. Rather than `cd`-ing around, each command builder accepts a working directory, and the crate handles it via:

- The `-chdir=<path>` global option (preferred, Terraform v0.14+)
- Or setting the subprocess working directory directly

The `Terraform` client struct can hold a default working directory, with per-command overrides available on any builder.

```rust
// Default working directory set on the client
let tf = Terraform::new()
    .working_dir("/tmp/my-infra")
    .build();

// Per-command override
let output = InitCommand::new()
    .working_dir("/tmp/other-infra")  // overrides client default
    .execute(&tf)
    .await?;
```

## Key Difference: State Management

Terraform maintains state, and the crate needs to be aware of this without *managing* it. The wrapper should:

- Support backend configuration passthrough (via `-backend-config` on init)
- Support `-state` and `-state-out` flags where applicable
- Parse state via `terraform show -json` into typed structs
- Stay out of the way — the crate wraps CLI behavior, it doesn't invent state management opinions

For ephemeral use cases, local state in the working directory is likely sufficient. Remote backends are a consumer concern.

## Command Set

### Tier 1: Core Lifecycle

These are the commands needed to go from zero to a running environment:

**`InitCommand`** — Prepare working directory, download providers/modules.

```rust
InitCommand::new()
    .backend_config("key", "value")
    .backend_config_file("backend.hcl")
    .upgrade()
    .reconfigure()
    .plugin_dir("/path/to/plugins")
    .input(false)
    .execute(&tf)
    .await?;
```

Key options: `-backend-config`, `-upgrade`, `-reconfigure`, `-plugin-dir`, `-input`, `-lock`, `-lock-timeout`

**`PlanCommand`** — Preview changes, optionally save plan file.

```rust
let plan = PlanCommand::new()
    .var("region", "us-west-2")
    .var_file("prod.tfvars")
    .out("tfplan")
    .target("module.vpc")
    .destroy()  // plan for destroy
    .json()     // structured output
    .execute(&tf)
    .await?;
```

Key options: `-var`, `-var-file`, `-out`, `-target`, `-destroy`, `-refresh-only`, `-replace`, `-lock`, `-parallelism`, `-json`, `-input`

**`ApplyCommand`** — Create or update infrastructure.

```rust
let result = ApplyCommand::new()
    .plan_file("tfplan")       // apply a saved plan
    .auto_approve()            // skip interactive prompt
    .var("region", "us-west-2")
    .target("module.vpc")
    .parallelism(10)
    .json()
    .execute(&tf)
    .await?;
```

Key options: `-auto-approve`, `-var`, `-var-file`, `-target`, `-parallelism`, `-lock`, `-json`, `-input`, plan file as positional arg

**`DestroyCommand`** — Tear down infrastructure.

```rust
DestroyCommand::new()
    .auto_approve()
    .target("module.vpc")
    .var("region", "us-west-2")
    .json()
    .execute(&tf)
    .await?;
```

Key options: Same as apply (destroy is effectively `apply -destroy`). Separate command struct for clarity and discoverability.

**`OutputCommand`** — Extract output values (e.g., connection strings, IPs).

```rust
// All outputs as JSON
let outputs: HashMap<String, OutputValue> = OutputCommand::new()
    .json()
    .execute(&tf)
    .await?;

// Single named output (raw value)
let conn_string: String = OutputCommand::new()
    .name("connection_string")
    .raw()
    .execute(&tf)
    .await?;
```

Key options: `-json`, `-raw`, `-no-color`, named output as positional arg

This is the critical command for any orchestration tool whose contract is "get me to a usable endpoint."

### Tier 2: Inspection & Validation

**`ValidateCommand`** — Check configuration syntax and consistency.

```rust
let result = ValidateCommand::new()
    .json()
    .execute(&tf)
    .await?;
// result.valid, result.error_count, result.diagnostics, etc.
```

**`ShowCommand`** — Inspect current state or a saved plan as structured data.

```rust
// Show current state
let state: StateRepresentation = ShowCommand::new()
    .json()
    .execute(&tf)
    .await?;

// Show a saved plan
let plan: PlanRepresentation = ShowCommand::new()
    .plan_file("tfplan")
    .json()
    .execute(&tf)
    .await?;
```

**`FmtCommand`** — Format configuration files.

```rust
FmtCommand::new()
    .recursive()
    .check()  // just check, don't modify
    .diff()
    .execute(&tf)
    .await?;
```

### Tier 3: State & Workspace Management (future)

These are useful for more advanced use cases but not required initially:

- **`WorkspaceCommand`** — `list`, `new`, `select`, `delete`, `show`
- **`StateCommand`** — `list`, `show`, `mv`, `rm`, `pull`, `push`
- **`ImportCommand`** — Associate existing resources with Terraform state
- **`TaintCommand`** / **`UntaintCommand`** — Mark resources for recreation
- **`ProvidersCommand`** — List required providers

### Tier 4: Utility (low priority)

- **`GraphCommand`** — Generate DOT dependency graph
- **`ConsoleCommand`** — Interactive expression evaluation (limited programmatic value)
- **`ForceUnlockCommand`** — Release stuck state locks
- **`LoginCommand`** / **`LogoutCommand`** — Registry authentication

## Core Types

### Terraform Client

```rust
pub struct Terraform {
    binary: PathBuf,          // default: "terraform" (found via PATH)
    working_dir: Option<PathBuf>,
    env: HashMap<String, String>,  // e.g., TF_LOG, TF_VAR_*, cloud credentials
    global_args: Vec<String>,      // args applied to every command
}

impl Terraform {
    pub fn new() -> TerraformBuilder { ... }

    /// Verify terraform is installed and return version info
    pub async fn version(&self) -> Result<VersionInfo, TerraformError> { ... }
}
```

### Command Trait

```rust
#[async_trait]
pub trait TerraformCommand {
    type Output;

    /// Build the argument list for this command
    fn args(&self) -> Vec<String>;

    /// Execute the command against a Terraform client
    async fn execute(&self, tf: &Terraform) -> Result<Self::Output, TerraformError>;
}
```

### Error Types

```rust
pub enum TerraformError {
    /// Terraform binary not found
    NotFound,

    /// Command exited with non-zero status
    CommandFailed {
        command: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
    },

    /// Failed to parse JSON output
    ParseError {
        source: serde_json::Error,
        raw_output: String,
    },

    /// IO error during subprocess execution
    IoError(std::io::Error),

    /// Terraform state is locked
    StateLocked {
        lock_id: String,
        info: String,
    },

    /// Plan contains no changes
    NoChanges,
}
```

### JSON Output Types

Terraform's `-json` output is well-documented. Key types to model:

```rust
/// terraform plan -json / terraform show -json (plan)
pub struct PlanRepresentation {
    pub format_version: String,
    pub terraform_version: String,
    pub planned_values: PlannedValues,
    pub resource_changes: Vec<ResourceChange>,
    pub output_changes: HashMap<String, OutputChange>,
}

/// terraform show -json (state)
pub struct StateRepresentation {
    pub format_version: String,
    pub terraform_version: String,
    pub values: StateValues,
}

/// terraform output -json
pub struct OutputValue {
    pub sensitive: bool,
    pub value: serde_json::Value,
    pub r#type: serde_json::Value,
}

/// terraform validate -json
pub struct ValidationResult {
    pub valid: bool,
    pub error_count: u32,
    pub warning_count: u32,
    pub diagnostics: Vec<Diagnostic>,
}

/// Streaming JSON lines from plan/apply with -json
pub struct JsonLogLine {
    pub level: String,       // "info", "warn", "error"
    pub message: String,
    pub r#type: String,      // "change_summary", "resource_drift", etc.
    pub hook: Option<serde_json::Value>,
}
```

## Global Options

All commands share certain Terraform global options. These can be set on the `Terraform` client (applied to every command) or per-command:

- `-chdir=<path>` — Working directory (primary mechanism for directory scoping)
- `-no-color` — Disable ANSI color (default on for programmatic use)
- `-input=false` — Disable interactive prompts (default for programmatic use)

Sensible defaults for programmatic use: `-no-color` and `-input=false` should be on by default, with opt-out available.

## Environment Variable Support

Terraform uses environment variables extensively. The crate should make it easy to pass these through:

- `TF_VAR_<name>` — Variable values (alternative to `-var`)
- `TF_LOG` — Logging level (TRACE, DEBUG, INFO, WARN, ERROR)
- `TF_DATA_DIR` — Override `.terraform` directory location
- `TF_PLUGIN_CACHE_DIR` — Shared plugin cache
- Cloud credentials: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `GOOGLE_CREDENTIALS`, etc.

```rust
let tf = Terraform::new()
    .working_dir("/tmp/infra")
    .env("TF_LOG", "INFO")
    .env("AWS_REGION", "us-west-2")
    .env_var("instance_type", "t3.medium")  // convenience: sets TF_VAR_instance_type
    .build();
```

## Feature Flags

Following docker-wrapper's approach:

```toml
[features]
default = ["json"]
json = ["serde", "serde_json"]   # typed JSON output parsing
streaming = ["tokio-stream"]      # streaming JSON log lines from plan/apply
templates = []                    # pre-built Terraform configurations (future, consumer-specific)
```

## Module Structure

```
src/
├── lib.rs              # Re-exports, Terraform client
├── error.rs            # TerraformError
├── command.rs          # TerraformCommand trait
├── commands/
│   ├── mod.rs
│   ├── init.rs         # InitCommand
│   ├── plan.rs         # PlanCommand
│   ├── apply.rs        # ApplyCommand
│   ├── destroy.rs      # DestroyCommand
│   ├── output.rs       # OutputCommand
│   ├── validate.rs     # ValidateCommand
│   ├── show.rs         # ShowCommand
│   ├── fmt.rs          # FmtCommand
│   ├── workspace.rs    # WorkspaceCommand (tier 3)
│   └── state.rs        # StateCommand (tier 3)
├── types/
│   ├── mod.rs
│   ├── plan.rs         # PlanRepresentation, ResourceChange, etc.
│   ├── state.rs        # StateRepresentation, StateValues, etc.
│   ├── output.rs       # OutputValue
│   ├── validation.rs   # ValidationResult, Diagnostic
│   └── version.rs      # VersionInfo
└── exec.rs             # Subprocess execution, shared by all commands
```

## Special Attention Items

### Auto-Approve for Programmatic Use

`apply` and `destroy` prompt interactively by default. For programmatic use, `-auto-approve` is almost always required. The crate should:

- Make it easy to set (`.auto_approve()`)
- Consider whether it should be the default for the wrapper (leaning yes, with `.interactive()` to opt out)
- Document the foot-gun clearly

### Streaming Output

`terraform plan` and `terraform apply` with `-json` produce streaming JSON lines (one per event) rather than a single JSON blob. The crate should support both:

- **Collected**: Wait for completion, return final result
- **Streaming**: Yield `JsonLogLine` events as they arrive (useful for progress reporting in orchestration tools)

### Destroy Safety

Given that a common use case is ephemeral environments, `destroy` will be called frequently. The crate itself shouldn't add safety rails beyond what Terraform provides — that's the consumer's job. But it should make the destroy lifecycle easy and reliable, including handling the case where destroy partially fails (common with cloud resources that have deletion protection or dependencies).

### Terraform Version Detection

The crate should detect the installed Terraform version and:

- Warn if below a minimum supported version (v1.0+ probably)
- Adapt behavior if needed (e.g., `-chdir` requires v0.14+)
- Expose version info to consumers

### Binary Discovery

Like docker-wrapper, support multiple ways to find the terraform binary:

1. Explicit path via builder
2. `TERRAFORM_PATH` environment variable
3. `terraform` on `PATH` (default)

## Open Questions

1. **OpenTofu compatibility**: Should the crate support OpenTofu as an alternative backend? The CLI is nearly identical. Could be as simple as swapping the binary path, but worth considering if there are divergences in JSON output formats.

2. **HCL generation**: Should the crate include helpers for generating `.tf` files programmatically, or is that out of scope? A consumer could adopt a "bring your own terraform" model where they supply their own configs, or a "managed" model where configs are generated. The wrapper probably shouldn't have opinions here, but it's worth considering.

3. **Plan file handling**: Terraform plan files are opaque binary blobs. Should the crate abstract over the "plan then apply" two-step, or keep them as separate operations? Leaning toward separate — let the consumer decide the workflow.

4. **Provider plugin caching**: For consumers spinning up many ephemeral environments, re-downloading providers every `init` is wasteful. Should the crate make `TF_PLUGIN_CACHE_DIR` easy to configure, or is that a consumer concern?

5. **Crate naming**: `terraform-wrapper` parallels `docker-wrapper` nicely. But is there a trademark concern with "terraform" in the crate name? HashiCorp has been litigious. Alternatives: `tf-wrapper`, `terrawrap`, `hcl-runner`.

## Example: Cloud Database Orchestration

For context, here's how a hypothetical tool might use terraform-wrapper to provision a cloud database cluster, extract connection info, and tear it down:

```rust
// Provision cloud infrastructure and return a connection endpoint
async fn deploy_cluster(config: &ClusterConfig) -> Result<ConnectionInfo, DeployError> {
    let tf = Terraform::new()
        .working_dir(&config.terraform_dir())
        .env("AWS_REGION", &config.region)
        .build();

    // Initialize (download providers)
    InitCommand::new()
        .execute(&tf)
        .await?;

    // Apply infrastructure
    ApplyCommand::new()
        .auto_approve()
        .var("node_count", &config.nodes.to_string())
        .var("instance_type", &config.instance_type)
        .execute(&tf)
        .await?;

    // Extract connection info — the whole point
    let host: String = OutputCommand::new()
        .name("cluster_host")
        .raw()
        .execute(&tf)
        .await?;

    let port: String = OutputCommand::new()
        .name("cluster_port")
        .raw()
        .execute(&tf)
        .await?;

    Ok(ConnectionInfo { host, port: port.parse()? })
}

// Teardown — no orphaned cloud resources
async fn destroy_cluster(config: &ClusterConfig) -> Result<(), DeployError> {
    let tf = Terraform::new()
        .working_dir(&config.terraform_dir())
        .build();

    DestroyCommand::new()
        .auto_approve()
        .execute(&tf)
        .await?;

    Ok(())
}
```

## Examples

The following examples assume you have Terraform configuration files (`.tf`) already written. This crate does not generate or manage Terraform configurations — it drives the Terraform CLI against configurations you provide. How you create and organize your `.tf` files is entirely up to you.

### Example 1: Basic Lifecycle — EC2 Instance

This mirrors the canonical Terraform getting-started tutorial: provision an EC2 instance, inspect it, and tear it down.

Given a directory `./infra/basic-ec2/` containing:

```hcl
# main.tf
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.region
}

resource "aws_instance" "example" {
  ami           = var.ami
  instance_type = var.instance_type

  tags = {
    Name = var.instance_name
  }
}

# variables.tf
variable "region" {
  default = "us-west-2"
}

variable "ami" {
  default = "ami-0c55b159cbfafe1f0"
}

variable "instance_type" {
  default = "t2.micro"
}

variable "instance_name" {
  default = "terraform-wrapper-example"
}

# outputs.tf
output "instance_id" {
  value = aws_instance.example.id
}

output "public_ip" {
  value = aws_instance.example.public_ip
}
```

The Rust code to drive the full lifecycle:

```rust
use terraform_wrapper::{
    Terraform, TerraformCommand,
    InitCommand, PlanCommand, ApplyCommand, OutputCommand, DestroyCommand,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create client pointed at the config directory
    let tf = Terraform::new()
        .working_dir("./infra/basic-ec2")
        .build();

    // 2. Initialize — downloads the AWS provider
    InitCommand::new()
        .execute(&tf)
        .await?;

    // 3. Plan — preview what will be created
    PlanCommand::new()
        .out("tfplan")
        .execute(&tf)
        .await?;

    // 4. Apply the saved plan
    ApplyCommand::new()
        .plan_file("tfplan")
        .auto_approve()
        .execute(&tf)
        .await?;

    // 5. Extract outputs
    let ip = OutputCommand::new()
        .name("public_ip")
        .raw()
        .execute(&tf)
        .await?;

    println!("Instance is running at: {ip}");

    // 6. Tear down when done
    DestroyCommand::new()
        .auto_approve()
        .execute(&tf)
        .await?;

    Ok(())
}
```

### Example 2: Parameterized Deployment with Variables

Override variables at apply time without modifying `.tf` files — useful when a single configuration serves multiple environments or instance sizes.

```rust
let tf = Terraform::new()
    .working_dir("./infra/basic-ec2")
    .build();

InitCommand::new().execute(&tf).await?;

ApplyCommand::new()
    .auto_approve()
    .var("region", "eu-west-1")
    .var("instance_type", "t3.large")
    .var("instance_name", "prod-worker-1")
    .execute(&tf)
    .await?;
```

Or use a `.tfvars` file:

```rust
ApplyCommand::new()
    .auto_approve()
    .var_file("environments/prod.tfvars")
    .execute(&tf)
    .await?;
```

### Example 3: Inspect State with JSON Output

Use `ShowCommand` to get a structured view of current state, useful for orchestration tools that need to reason about what's deployed.

```rust
let state: StateRepresentation = ShowCommand::new()
    .json()
    .execute(&tf)
    .await?;

for resource in &state.values.root_module.resources {
    println!("{}: {} ({})",
        resource.address,
        resource.name,
        resource.r#type,
    );
}
```

### Example 4: Validate Before Apply

Check that configuration is syntactically and semantically valid before committing to a plan/apply cycle — useful in CI pipelines.

```rust
let tf = Terraform::new()
    .working_dir("./infra/basic-ec2")
    .build();

InitCommand::new().execute(&tf).await?;

let result = ValidateCommand::new()
    .json()
    .execute(&tf)
    .await?;

if !result.valid {
    for diag in &result.diagnostics {
        eprintln!("[{}] {}: {}", diag.severity, diag.summary, diag.detail);
    }
    std::process::exit(1);
}

println!("Configuration is valid, proceeding with apply...");
```

### Example 5: Multiple Working Directories

Manage independent stacks from the same process. Each working directory has its own state, providers, and configuration.

```rust
let network = Terraform::new()
    .working_dir("./infra/network")
    .build();

let compute = Terraform::new()
    .working_dir("./infra/compute")
    .build();

// Stand up network first
InitCommand::new().execute(&network).await?;
ApplyCommand::new().auto_approve().execute(&network).await?;

// Extract VPC ID to feed into compute stack
let vpc_id = OutputCommand::new()
    .name("vpc_id")
    .raw()
    .execute(&network)
    .await?;

// Stand up compute, passing network output as a variable
InitCommand::new().execute(&compute).await?;
ApplyCommand::new()
    .auto_approve()
    .var("vpc_id", &vpc_id)
    .execute(&compute)
    .await?;

// Tear down in reverse order
DestroyCommand::new().auto_approve().execute(&compute).await?;
DestroyCommand::new().auto_approve().execute(&network).await?;
```

### Example 6: Environment Variables for Credentials

Pass cloud credentials without embedding them in code or `.tf` files.

```rust
let tf = Terraform::new()
    .working_dir("./infra/basic-ec2")
    .env("AWS_ACCESS_KEY_ID", &std::env::var("AWS_ACCESS_KEY_ID")?)
    .env("AWS_SECRET_ACCESS_KEY", &std::env::var("AWS_SECRET_ACCESS_KEY")?)
    .env("AWS_REGION", "us-west-2")
    .build();
```

Or for GCP:

```rust
let tf = Terraform::new()
    .working_dir("./infra/gcp-instance")
    .env("GOOGLE_CREDENTIALS", &std::fs::read_to_string("service-account.json")?)
    .env("GOOGLE_PROJECT", "my-project-id")
    .build();
```

### A Note on Terraform Configuration Files

This crate is deliberately agnostic about how `.tf` files are created and managed. Common patterns include:

- **Static files checked into version control** — the most common approach. Write your `.tf` files by hand, commit them alongside your application code, and point `terraform-wrapper` at the directory.

- **Template-based generation** — use a templating engine (e.g., Tera, Askama, or even `format!`) to generate `.tf` files from parameters before running init/apply. Useful when the same infrastructure pattern needs many variations.

- **Bundled configurations** — ship `.tf` files as embedded resources in your binary (via `include_str!` or `rust-embed`) and write them to a temp directory at runtime. Good for CLI tools that need to be self-contained.

- **Bring Your Own Terraform (BYOT)** — let the user point your tool at their own `.tf` files. The tool provides the orchestration lifecycle; the user provides the infrastructure definition.

The crate doesn't prescribe any of these. It drives `terraform init/plan/apply/destroy/output` against whatever directory you point it at.

## Implementation Priority

1. `Terraform` client struct + binary discovery + version detection
2. `TerraformCommand` trait + subprocess execution (`exec.rs`)
3. `TerraformError` enum
4. Tier 1 commands: `init`, `plan`, `apply`, `destroy`, `output`
5. JSON output types for plan, state, output, validation
6. Tier 2 commands: `validate`, `show`, `fmt`
7. Streaming JSON support
8. Tier 3+ as needed
