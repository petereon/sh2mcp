# sh2mcp

Wrap any shell command as an [MCP](https://modelcontextprotocol.io) tool — no code required.

```
sh2mcp --tool "disk" --description "Check disk usage" --command "df -h"
```

## Installation

```bash
cargo install --git https://github.com/petervyboch/sh2mcp
```

Or build from source:

```bash
cargo build --release
# binary at target/release/sh2mcp
```

## Usage

```
sh2mcp [--tool <name> --description <desc> --command <cmd>]...
```

Each `--tool`/`--description`/`--command` triplet registers one MCP tool. Repeat for multiple tools.

### Static commands (no model input)

```bash
sh2mcp \
  --tool "date"       --description "Print the current date and time" --command "date" \
  --tool "disk-usage" --description "Show disk usage"                 --command "df -h" \
  --tool "processes"  --description "List running processes"          --command "ps aux"
```

### Dynamic commands with `{placeholder}`

Wrap `{name}` in curly braces in `--command` to create a named string parameter the model fills in at call time.

```bash
sh2mcp \
  --tool "search-files" \
  --description "Find files matching a pattern under a directory" \
  --command "find {directory} -name {pattern}"
```

When the model calls `search-files`, it supplies `directory` and `pattern` as arguments.

Multiple placeholders, pipes, and shell features all work:

```bash
sh2mcp \
  --tool "grep" \
  --description "Search for a regex pattern in a file and return matching lines" \
  --command "grep -n {pattern} {file}"

sh2mcp \
  --tool "count-lines" \
  --description "Count lines in a file matching a pattern" \
  --command "grep -c {pattern} {file}"

sh2mcp \
  --tool "git-log" \
  --description "Show recent git log for a repo" \
  --command "git -C {repo_path} log --oneline -{count}"
```

## Integrating with Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "my-tools": {
      "command": "sh2mcp",
      "args": [
        "--tool", "grep",
        "--description", "Search for a regex pattern in a file",
        "--command", "grep -n {pattern} {file}",
        "--tool", "disk-usage",
        "--description", "Show disk usage",
        "--command", "df -h"
      ]
    }
  }
}
```

## Integrating with any MCP client

`sh2mcp` speaks MCP over stdio — point any MCP-compatible client at the binary.

Test interactively with the [MCP Inspector](https://github.com/modelcontextprotocol/inspector):

```bash
npx @modelcontextprotocol/inspector sh2mcp \
  --tool "echo" --description "Echo a message" --command "echo {message}"
```

> [!CAUTION]
> This currently doesn't work becuase of a bug in the way inspector parses arguments (https://github.com/modelcontextprotocol/inspector/pull/1162)

## Behaviour notes

- **Non-zero exit codes** are returned as `isError: true` with stdout, stderr, and the exit code included so the model can reason about failures.
- **Placeholder values** are substituted raw into `sh -c`. Avoid passing untrusted user input as argument values.
- **Logging** goes to stderr, keeping stdout clean for the JSON-RPC transport.
