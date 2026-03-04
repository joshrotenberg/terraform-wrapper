use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for generating a visual dependency graph in DOT format.
///
/// Produces a representation of the dependency graph between Terraform
/// resources, suitable for rendering with Graphviz.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::graph::GraphCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// let output = GraphCommand::new()
///     .draw_cycles()
///     .execute(&tf)
///     .await?;
/// // output.stdout contains DOT format graph
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct GraphCommand {
    graph_type: Option<String>,
    plan_file: Option<String>,
    draw_cycles: bool,
    raw_args: Vec<String>,
}

impl GraphCommand {
    /// Create a new graph command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the graph type (`-type=plan|plan-refresh-only|plan-destroy|apply`).
    #[must_use]
    pub fn graph_type(mut self, graph_type: &str) -> Self {
        self.graph_type = Some(graph_type.to_string());
        self
    }

    /// Use a saved plan file (`-plan=<file>`).
    ///
    /// Automatically sets the graph type to `apply`.
    #[must_use]
    pub fn plan_file(mut self, path: &str) -> Self {
        self.plan_file = Some(path.to_string());
        self
    }

    /// Highlight circular dependencies (`-draw-cycles`).
    #[must_use]
    pub fn draw_cycles(mut self) -> Self {
        self.draw_cycles = true;
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for GraphCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["graph".to_string()];
        if let Some(ref graph_type) = self.graph_type {
            args.push(format!("-type={graph_type}"));
        }
        if let Some(ref plan_file) = self.plan_file {
            args.push(format!("-plan={plan_file}"));
        }
        if self.draw_cycles {
            args.push("-draw-cycles".to_string());
        }
        args.extend(self.raw_args.clone());
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<CommandOutput> {
        exec::run_terraform(tf, self.args()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args() {
        let cmd = GraphCommand::new();
        assert_eq!(cmd.args(), vec!["graph"]);
    }

    #[test]
    fn draw_cycles() {
        let cmd = GraphCommand::new().draw_cycles();
        assert_eq!(cmd.args(), vec!["graph", "-draw-cycles"]);
    }

    #[test]
    fn graph_type() {
        let cmd = GraphCommand::new().graph_type("plan-destroy");
        assert_eq!(cmd.args(), vec!["graph", "-type=plan-destroy"]);
    }

    #[test]
    fn plan_file() {
        let cmd = GraphCommand::new().plan_file("tfplan");
        assert_eq!(cmd.args(), vec!["graph", "-plan=tfplan"]);
    }

    #[test]
    fn all_options() {
        let cmd = GraphCommand::new()
            .graph_type("apply")
            .plan_file("tfplan")
            .draw_cycles();
        let args = cmd.args();
        assert_eq!(args[0], "graph");
        assert!(args.contains(&"-type=apply".to_string()));
        assert!(args.contains(&"-plan=tfplan".to_string()));
        assert!(args.contains(&"-draw-cycles".to_string()));
    }
}
