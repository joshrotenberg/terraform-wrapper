use crate::Terraform;
use crate::error::Result;

/// Trait implemented by all Terraform command builders.
///
/// Each command implements [`args`](TerraformCommand::args) to produce its CLI
/// arguments and [`execute`](TerraformCommand::execute) to run against a
/// [`Terraform`] client.
pub trait TerraformCommand: Send + Sync {
    /// The output type produced by this command.
    type Output: Send;

    /// Build the argument list for this command.
    ///
    /// Returns the subcommand name followed by all flags and options.
    /// For example: `["init", "-upgrade", "-backend-config=key=value"]`.
    fn args(&self) -> Vec<String>;

    /// Execute this command against the given Terraform client.
    fn execute(
        &self,
        tf: &Terraform,
    ) -> impl std::future::Future<Output = Result<Self::Output>> + Send;
}
