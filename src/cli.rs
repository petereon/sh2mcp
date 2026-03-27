use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "sh2mcp", about = "Wrap shell commands as MCP tools served over stdio")]
pub struct Cli {
    /// Tool name (repeat for each tool)
    #[arg(long = "tool", action = clap::ArgAction::Append, required = true)]
    pub tools: Vec<String>,

    /// Tool description shown to the model (repeat for each tool)
    #[arg(long = "description", action = clap::ArgAction::Append, required = true)]
    pub descriptions: Vec<String>,

    /// Shell command template; use {placeholder} for model-supplied args (repeat for each tool)
    #[arg(long = "command", action = clap::ArgAction::Append, required = true)]
    pub commands: Vec<String>,
}

impl Cli {
    pub fn validate(&self) -> anyhow::Result<()> {
        let n = self.tools.len();
        anyhow::ensure!(
            self.descriptions.len() == n && self.commands.len() == n,
            "--tool, --description, and --command counts must match (got {}, {}, {})",
            n,
            self.descriptions.len(),
            self.commands.len(),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli(tools: &[&str], descriptions: &[&str], commands: &[&str]) -> Cli {
        Cli {
            tools: tools.iter().map(|s| s.to_string()).collect(),
            descriptions: descriptions.iter().map(|s| s.to_string()).collect(),
            commands: commands.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn validate_matching_counts_passes() {
        let c = cli(&["t1", "t2"], &["d1", "d2"], &["cmd1", "cmd2"]);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validate_single_tool_passes() {
        let c = cli(&["t"], &["d"], &["cmd"]);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validate_missing_description_fails() {
        let c = cli(&["t1", "t2"], &["d1"], &["cmd1", "cmd2"]);
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("must match"));
    }

    #[test]
    fn validate_missing_command_fails() {
        let c = cli(&["t1", "t2"], &["d1", "d2"], &["cmd1"]);
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("must match"));
    }

    #[test]
    fn validate_extra_command_fails() {
        let c = cli(&["t"], &["d"], &["cmd1", "cmd2"]);
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("must match"));
    }
}
