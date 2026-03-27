use std::sync::Arc;

use rmcp::{
    ErrorData as McpError,
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Content, Implementation,
        ListToolsResult, PaginatedRequestParams, RawContent, ServerCapabilities,
        ServerInfo, Tool,
    },
    service::{RequestContext, RoleServer},
};
use serde_json::{json, Map, Value};

use crate::executor::run_shell;
use crate::tool_def::ToolDef;

#[derive(Clone)]
pub struct ShellHandler {
    tools: Arc<Vec<ToolDef>>,
}

impl ShellHandler {
    pub fn new(tools: Vec<ToolDef>) -> Self {
        Self { tools: Arc::new(tools) }
    }

    fn build_input_schema(params: &[String]) -> Arc<Map<String, Value>> {
        let mut properties = Map::new();
        for p in params {
            properties.insert(p.clone(), json!({"type": "string"}));
        }
        let schema = json!({
            "type": "object",
            "properties": properties,
            "required": params,
        });
        Arc::new(match schema {
            Value::Object(m) => m,
            _ => unreachable!(),
        })
    }

    fn to_mcp_tool(def: &ToolDef) -> Tool {
        Tool::new(
            def.name.clone(),
            def.description.clone(),
            Self::build_input_schema(&def.params),
        )
    }

    fn text_content(text: String) -> Content {
        Content::new(RawContent::text(text), None)
    }
}

impl ServerHandler for ShellHandler {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = Implementation::from_build_env();
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let mcp_tools: Vec<Tool> = self.tools.iter().map(Self::to_mcp_tool).collect();
        Ok(ListToolsResult::with_all_items(mcp_tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let def = self.tools.iter()
            .find(|t| t.name == request.name.as_ref())
            .ok_or_else(|| McpError::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            ))?;

        let args = request.arguments.unwrap_or_default();

        let command = def.render(&args).map_err(|e| {
            McpError::invalid_params(e.to_string(), None)
        })?;

        let result = run_shell(&command).await.map_err(|e| {
            McpError::internal_error(format!("execution failed: {e}"), None)
        })?;

        if result.exit_code == 0 {
            let mut text = result.stdout;
            if !result.stderr.is_empty() {
                text.push_str(&result.stderr);
            }
            Ok(CallToolResult::success(vec![Self::text_content(text)]))
        } else {
            let text = format!(
                "exit code: {}\nstdout: {}\nstderr: {}",
                result.exit_code, result.stdout, result.stderr
            );
            Ok(CallToolResult::error(vec![Self::text_content(text)]))
        }
    }
}
