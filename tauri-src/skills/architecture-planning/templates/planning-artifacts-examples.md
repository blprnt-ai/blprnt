# Planning Artifacts Examples (Detailed)

This document provides detailed examples for single-level Plan artifacts.

The examples are deliberately verbose to show the level of clarity expected in real plans.

---

## Plan Example

**Item**  
Implement MCP client (tool discovery + invocation) across CLI and API layers.

**Outcome Goal**  
Users can list MCP tools, inspect input schemas, and invoke tools reliably from both
CLI and programmatic interfaces without requiring direct knowledge of MCP internals.

**Scope Boundaries**
- In scope:
  - Client API for tool discovery and invocation
  - CLI commands to list tools and call tools
  - Input schema parsing and validation
  - Basic authentication hooks for MCP servers
  - Clear error mapping and user-readable failures
- Out of scope:
  - GUI integration
  - Automatic retries/backoff policies
  - Multi-tenant tool isolation
  - Full protocol version negotiation

**Success Metrics**
- Tool list latency under 300ms for a typical MCP server
- CLI command coverage for list and call flows
- End-to-end demo using a local MCP server

**Risks**
- Risk: Tool schemas vary by server and may be loosely specified.  
  Mitigation: Tolerant parsing with validation warnings and fallback handling.
- Risk: Authentication mechanisms differ by provider.  
  Mitigation: Define a pluggable auth adapter interface and ship with a default.

---

## Plan Example (Detailed)

**Item**  
Implement MCP client (tool discovery + invocation) across CLI and API layers.

**Outcome Goal**  
Users can list MCP tools, inspect input schemas, and invoke tools reliably from both
CLI and programmatic interfaces without requiring direct knowledge of MCP internals.

**Scope Boundaries**
- In scope:
  - Client API for tool discovery and invocation
  - CLI commands to list tools and call tools
  - Input schema parsing and validation
  - Basic authentication hooks for MCP servers
  - Clear error mapping and user-readable failures
- Out of scope:
  - GUI integration
  - Automatic retries/backoff policies
  - Multi-tenant tool isolation
  - Full protocol version negotiation

**Success Metrics**
- Tool list latency under 300ms for a typical MCP server
- CLI command coverage for list and call flows
- End-to-end demo using a local MCP server

**Assumptions**
- MCP servers expose `GET /tools` and `POST /tools/{name}/call`.
- Schemas are JSON Schema compatible enough for validation.
- CLI runtime already supports JSON input parsing.

**Dependencies**
- MCP server definitions and schema access
- CLI command router
- Existing HTTP client and logging utilities

**Interfaces**
- `McpClient::list_tools() -> Vec<ToolSummary>`
- `McpClient::get_tool_schema(name) -> ToolSchema`
- `McpClient::call_tool(name, args) -> ToolResult`
- CLI: `mcp tools`, `mcp tool <name>`, `mcp call <name> --args <json>`

**Acceptance Criteria**
- Tool discovery returns name, description, and input schema.
- Tool invocation validates args against schema and returns structured output.
- CLI can list tools, show a tool schema, and invoke a tool with JSON args.
- Error messages include the tool name and a user-facing cause.

**Risks**
- Schema validation rejects legitimate but loosely-typed inputs.  
  Mitigation: Allow a strict/lenient validation mode with warnings.
- CLI output becomes inconsistent across error types.  
  Mitigation: Centralize formatting in a shared renderer.

**Verification Plan**
- Unit tests for schema parsing and validation
- Unit tests for client invocation paths using mocked MCP server
- CLI smoke tests for list and call flows
