import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { shouldRegisterTool, bridgeTools, type McpClientLike, type OpenClawApi, type OpenClawToolRegistration, type BridgeOptions } from "../tool-bridge.js";

// ---------------------------------------------------------------------------
// shouldRegisterTool
// ---------------------------------------------------------------------------

describe("shouldRegisterTool", () => {
  it("allows read tools by default (no mutations)", () => {
    assert.equal(shouldRegisterTool("get_stats", {}), true);
    assert.equal(shouldRegisterTool("get_config", {}), true);
  });

  it("blocks mutation tools by default", () => {
    assert.equal(shouldRegisterTool("x_post_tweet", {}), false);
    assert.equal(shouldRegisterTool("x_like_tweet", {}), false);
    assert.equal(shouldRegisterTool("approve_all", {}), false);
  });

  it("allows non-mutating composite tools by default", () => {
    assert.equal(shouldRegisterTool("find_reply_opportunities", {}), true);
    assert.equal(shouldRegisterTool("draft_replies_for_candidates", {}), true);
  });

  it("blocks policy-gated composite tools by default", () => {
    assert.equal(shouldRegisterTool("propose_and_queue_replies", {}), false);
  });

  it("allows ops tools by default", () => {
    assert.equal(shouldRegisterTool("health_check", {}), true);
    assert.equal(shouldRegisterTool("get_mode", {}), true);
  });

  it("allows all tools when enableMutations is true", () => {
    const opts: BridgeOptions = { enableMutations: true };
    assert.equal(shouldRegisterTool("x_post_tweet", opts), true);
    assert.equal(shouldRegisterTool("x_like_tweet", opts), true);
    assert.equal(shouldRegisterTool("propose_and_queue_replies", opts), true);
    assert.equal(shouldRegisterTool("get_stats", opts), true);
  });

  it("restricts to named tools only when allowedTools is set", () => {
    const opts: BridgeOptions = { allowedTools: ["get_stats", "health_check"] };
    assert.equal(shouldRegisterTool("get_stats", opts), true);
    assert.equal(shouldRegisterTool("health_check", opts), true);
    assert.equal(shouldRegisterTool("get_config", opts), false);
  });

  it("filters by allowCategories", () => {
    const opts: BridgeOptions = { allowCategories: ["read"] };
    assert.equal(shouldRegisterTool("get_stats", opts), true);
    assert.equal(shouldRegisterTool("health_check", opts), false); // ops
    assert.equal(shouldRegisterTool("find_reply_opportunities", opts), false); // composite
  });

  it("filters by denyCategories", () => {
    const opts: BridgeOptions = { denyCategories: ["composite"] };
    assert.equal(shouldRegisterTool("get_stats", opts), true);
    assert.equal(shouldRegisterTool("find_reply_opportunities", opts), false);
    assert.equal(shouldRegisterTool("health_check", opts), true);
  });

  it("filters by maxRiskLevel: low blocks medium and high", () => {
    const opts: BridgeOptions = { maxRiskLevel: "low", enableMutations: true };
    assert.equal(shouldRegisterTool("get_stats", opts), true);       // low
    assert.equal(shouldRegisterTool("reject_item", opts), true);     // low
    assert.equal(shouldRegisterTool("x_like_tweet", opts), false);   // medium
    assert.equal(shouldRegisterTool("x_post_tweet", opts), false);   // high
  });

  it("filters by maxRiskLevel: medium blocks only high", () => {
    const opts: BridgeOptions = { maxRiskLevel: "medium", enableMutations: true };
    assert.equal(shouldRegisterTool("x_like_tweet", opts), true);    // medium
    assert.equal(shouldRegisterTool("x_post_tweet", opts), false);   // high
    assert.equal(shouldRegisterTool("approve_all", opts), false);    // high
  });

  it("passes unknown tools by default (forward-compatible)", () => {
    assert.equal(shouldRegisterTool("future_tool_xyz", {}), true);
  });

  it("composes multiple filters correctly", () => {
    const opts: BridgeOptions = {
      enableMutations: true,
      allowCategories: ["read", "mutation"],
      maxRiskLevel: "medium",
    };
    assert.equal(shouldRegisterTool("get_stats", opts), true);       // read, low
    assert.equal(shouldRegisterTool("x_like_tweet", opts), true);    // mutation, medium
    assert.equal(shouldRegisterTool("x_post_tweet", opts), false);   // mutation, high → blocked by risk
    assert.equal(shouldRegisterTool("health_check", opts), false);   // ops → blocked by category
  });
});

// ---------------------------------------------------------------------------
// bridgeTools
// ---------------------------------------------------------------------------

describe("bridgeTools", () => {
  function makeMockClient(tools: Array<{ name: string; description?: string }>): McpClientLike {
    return {
      async listTools() {
        return tools.map((t) => ({
          name: t.name,
          description: t.description,
          inputSchema: { type: "object", properties: {} },
        }));
      },
      async callTool(_name: string, _args: Record<string, unknown>) {
        return {
          content: [{ type: "text", text: JSON.stringify({ success: true, data: { ok: true } }) }],
        };
      },
    };
  }

  function makeMockApi(): OpenClawApi & { tools: OpenClawToolRegistration[] } {
    const tools: OpenClawToolRegistration[] = [];
    return {
      tools,
      registerTool(tool: OpenClawToolRegistration) {
        tools.push(tool);
      },
    };
  }

  it("registers only non-mutation tools by default", async () => {
    const client = makeMockClient([
      { name: "get_stats", description: "Get stats" },
      { name: "x_post_tweet", description: "Post a tweet" },
      { name: "health_check", description: "Health check" },
    ]);
    const api = makeMockApi();

    const count = await bridgeTools(client, api, {});
    assert.equal(count, 2);
    assert.deepEqual(
      api.tools.map((t) => t.name),
      ["tuitbot_get_stats", "tuitbot_health_check"],
    );
  });

  it("prefixes tool names with tuitbot_", async () => {
    const client = makeMockClient([{ name: "get_stats" }]);
    const api = makeMockApi();

    await bridgeTools(client, api, {});
    assert.equal(api.tools[0]?.name, "tuitbot_get_stats");
  });

  it("includes category tag in description", async () => {
    const client = makeMockClient([{ name: "get_stats", description: "Get stats" }]);
    const api = makeMockApi();

    await bridgeTools(client, api, {});
    assert.ok(api.tools[0]?.description.startsWith("[read]"));
  });

  it("includes policy-gated tag for policy-gated tools", async () => {
    const client = makeMockClient([{ name: "x_post_tweet", description: "Post a tweet" }]);
    const api = makeMockApi();

    await bridgeTools(client, api, { enableMutations: true });
    assert.ok(api.tools[0]?.description.includes("policy-gated"));
  });

  it("execute wrapper returns EnrichedToolResult", async () => {
    const client = makeMockClient([{ name: "get_stats" }]);
    const api = makeMockApi();

    await bridgeTools(client, api, {});
    const result = await api.tools[0]!.execute({});
    assert.equal(result.success, true);
    assert.deepEqual(result.data, { ok: true });
  });

  it("uses fallback description for tools without description", async () => {
    const client = makeMockClient([{ name: "get_stats" }]);
    const api = makeMockApi();

    await bridgeTools(client, api, {});
    assert.ok(api.tools[0]?.description.includes("Tuitbot MCP tool: get_stats"));
  });

  it("registers all tools when enableMutations is true", async () => {
    const client = makeMockClient([
      { name: "get_stats" },
      { name: "x_post_tweet" },
      { name: "x_like_tweet" },
      { name: "propose_and_queue_replies" },
    ]);
    const api = makeMockApi();

    const count = await bridgeTools(client, api, { enableMutations: true });
    assert.equal(count, 4);
  });
});
