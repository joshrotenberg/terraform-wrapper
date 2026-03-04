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

    /// Whether this command supports the `-input=false` flag.
    ///
    /// Defaults to `false`. Commands that accept `-input` (init, plan, apply,
    /// destroy, import, refresh) should override this to return `true`.
    fn supports_input(&self) -> bool {
        false
    }

    /// Build the argument list with `-input=false` injected when appropriate.
    ///
    /// Calls [`args`](TerraformCommand::args) and, if `tf.no_input` is set
    /// and [`supports_input`](TerraformCommand::supports_input) returns `true`,
    /// inserts `-input=false` after the subcommand name.
    fn prepare_args(&self, tf: &Terraform) -> Vec<String> {
        let mut args = self.args();
        if tf.no_input && self.supports_input() {
            args.insert(1, "-input=false".to_string());
        }
        args
    }

    /// Execute this command against the given Terraform client.
    fn execute(
        &self,
        tf: &Terraform,
    ) -> impl std::future::Future<Output = Result<Self::Output>> + Send;
}
