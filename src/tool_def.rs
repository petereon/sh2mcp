use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub command_template: String,
    /// Unique placeholder names in order of first appearance.
    pub params: Vec<String>,
}

fn placeholder_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{(\w+)\}").unwrap())
}

impl ToolDef {
    pub fn new(name: String, description: String, command_template: String) -> Self {
        let re = placeholder_re();
        let mut params: Vec<String> = Vec::new();
        for cap in re.captures_iter(&command_template) {
            let p = cap[1].to_string();
            if !params.contains(&p) {
                params.push(p);
            }
        }
        Self { name, description, command_template, params }
    }

    /// Substitute `{placeholder}` tokens with values from `args`.
    /// Returns an error if a required placeholder has no corresponding value.
    pub fn render(&self, args: &serde_json::Map<String, serde_json::Value>) -> anyhow::Result<String> {
        let missing: Vec<&str> = self.params.iter()
            .filter(|p| !args.contains_key(p.as_str()))
            .map(|p| p.as_str())
            .collect();
        if !missing.is_empty() {
            anyhow::bail!("missing arguments: {}", missing.join(", "));
        }

        let re = placeholder_re();
        let result = re.replace_all(&self.command_template, |caps: &regex::Captures| {
            // NOTE: placeholder values are substituted raw into sh -c; callers are
            // responsible for ensuring values do not contain unintended shell metacharacters.
            args[&caps[1]].as_str().unwrap_or("").to_string()
        });
        Ok(result.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(pairs: &[(&str, &str)]) -> serde_json::Map<String, serde_json::Value> {
        pairs.iter().map(|(k, v)| (k.to_string(), json!(v))).collect()
    }

    fn def(cmd: &str) -> ToolDef {
        ToolDef::new("t".into(), "d".into(), cmd.into())
    }

    // --- placeholder extraction ---

    #[test]
    fn no_placeholders() {
        let t = def("df -h");
        assert!(t.params.is_empty());
    }

    #[test]
    fn single_placeholder() {
        let t = def("echo {message}");
        assert_eq!(t.params, ["message"]);
    }

    #[test]
    fn multiple_placeholders() {
        let t = def("grep {pattern} {file}");
        assert_eq!(t.params, ["pattern", "file"]);
    }

    #[test]
    fn duplicate_placeholders_deduplicated() {
        let t = def("sed 's/{old}/{new}/g' {old}");
        // "old" appears twice but should only be listed once, in order of first appearance
        assert_eq!(t.params, ["old", "new"]);
    }

    // --- render ---

    #[test]
    fn render_no_placeholders() {
        let t = def("df -h");
        let result = t.render(&args(&[])).unwrap();
        assert_eq!(result, "df -h");
    }

    #[test]
    fn render_substitutes_values() {
        let t = def("grep {pattern} {file}");
        let result = t.render(&args(&[("pattern", "foo"), ("file", "main.rs")])).unwrap();
        assert_eq!(result, "grep foo main.rs");
    }

    #[test]
    fn render_repeated_placeholder_substituted_everywhere() {
        let t = def("sed 's/{old}/{new}/g' {old}");
        let result = t.render(&args(&[("old", "bar"), ("new", "baz")])).unwrap();
        assert_eq!(result, "sed 's/bar/baz/g' bar");
    }

    #[test]
    fn render_missing_arg_returns_error() {
        let t = def("grep {pattern} {file}");
        let err = t.render(&args(&[("pattern", "foo")])).unwrap_err();
        assert!(err.to_string().contains("missing arguments"));
        assert!(err.to_string().contains("file"));
    }

    #[test]
    fn render_extra_args_are_ignored() {
        let t = def("echo {message}");
        let result = t.render(&args(&[("message", "hi"), ("unused", "x")])).unwrap();
        assert_eq!(result, "echo hi");
    }
}
