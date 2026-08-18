# MCP context — stateless spec, disclosure, tutors

How an agent should drive `nbcad-mcp`, what the **2026-07-28** spec
actually means here, and how print-kit-style tutors plug in.

Day-to-day loop: [agent-mcp.md](../agent-mcp.md).
Wire manual: [MCP_2026.md](MCP_2026.md).
Invariants: [STEERABLE_MCP.md](STEERABLE_MCP.md).

## How the process works today

`nbcad-mcp` is a **Rust stdio** server (`mcp-server`). One process owns
**one in-process CAD document**. That is application state, not an MCP
session.

```text
Cursor / CLI  --JSON-RPC lines-->  nbcad-mcp.exe
                                      |
                                      +-- host::handle(SketchManager, method, payload)
                                      +-- OcctKernel (solids, check, export)
                                      +-- optional cad_attach → %NBCAD_SESSION_DIR%/<uuid>/model.json
```

- **Stdio (default):** stdin lines in, stdout JSON-RPC only, logs on stderr.
  `node scripts/nbcad-cli.mjs` is the same path the exam uses.
- **Bus (`NBCAD_MCP_TRANSPORT=bus-jsonl`):** same JSON-RPC wrapped in
  `nbcad.mcp-bus.v1` for Kafka/MQTT/NATS connectors. Still one document
  per worker. [mcp-message-bus.md](../mcp-message-bus.md).
- **Live UI:** `cad_list_sessions` → `cad_attach` `{mode:live}` claims
  `writer=mcp` and writebacks `model.json`. Headless goldens do not attach.

There is **no** protocol-level session id. The document *is* the handle.

## Optimal stateless MCP (2026-07-28)

The spec dropped `initialize` / `notifications/initialized` /
`Mcp-Session-Id`. Every request is self-describing.

1. Optional `server/discover` — capabilities, instructions, versions.
2. Every later request carries `params._meta`:

```json
{
  "io.modelcontextprotocol/protocolVersion": "2026-07-28",
  "io.modelcontextprotocol/clientCapabilities": {},
  "io.modelcontextprotocol/clientInfo": { "name": "your-client", "version": "0" }
}
```

`clientCapabilities` may be omitted (treated as `{}`). A non-object is
rejected. `clientInfo` is recommended.

3. `tools/list` / `tools/call` / `resources/*` / `prompts/*` — no handshake.
4. List results are `resultType: "complete"` plus `ttlMs` / `cacheScope`.
5. Dynamic tools: `subscriptions/listen` `{notifications:{toolsListChanged:true}}`
   then `tools/list` when `notifications/tools/list_changed` arrives.
   The spec's listen is a **long-lived HTTP stream** that starts with
   `notifications/subscriptions/acknowledged`. Holding that JSON-RPC
   request open on stdio hangs Cursor, so we ack + complete immediately
   and keep **pushing** `notifications/tools/list_changed` on stdout
   (Jack §2), tagged with `subscriptionId` if you listened.

**Do not wait** for `notifications/initialized`. That notification is
gone. The print-kit CLI used to hang there.

Stateless protocol ≠ stateless CAD. The spec's advice: mint an explicit
handle the model can pass around. Here the handle is the document in
this process (and, if attached, the session UUID).

## Dynamic disclosure (like the UI)

Same idea as switching Model / Drawing / Assembly in the desktop:

| Pack | Mutation tools |
|------|----------------|
| sketch | `sketch_*` |
| solid | extrude / revolve / sweep / loft / rib |
| modify | fillet / chamfer / hole |
| body_ops | shell, move/copy, patterns, combine |
| assembly | every `assembly_*` including motion + interference |
| drawing | sheets, HLR, DXF/SVG |
| print | 3MF / materials |
| inspect | tessellate (+ check is on the spine) |

`cad_set_focus` / `cad_set_workspace` advertise the active pack, keep
the last **3** packs soft for 60 s, and schedule
`notifications/tools/list_changed`.

**Guidance, not a jail.** Out-of-focus tools stay callable. A hidden
call re-promotes the pack. `full_static` or `cad_list_all_tools` if the
client never refreshes the list.

A **new process starts in sketch**, with **solid** already soft. First
`tools/list` includes `sketch_begin` and `solid_extrude` — no
`cad_set_focus` round-trip before the first profile. After
`sketch_finish`, auto-hint flips active to solid. After any
`assembly_*` call, assembly becomes active. `cad_set_workspace`
`drawing|assembly` matches the desktop switcher.

### Always-on spine

These stay in `tools/list` in every focus so an agent can recover
without hunting:

`cad_agent_guidance` · `cad_set_focus` · `cad_set_workspace` ·
`cad_list_all_tools` · `solid_scene` · `solid_check` ·
`assembly_document` · `assembly_solution` · `assembly_create_component` ·
`assembly_create_joint` · `assembly_set_joint_motion` ·
`assembly_evaluate_motion_study` · `assembly_interference_check` ·
`cad_drawing_document` · `cad_drawing_create_sheet` ·
`cad_undo` · `cad_redo`

There is **no 3D sketch** tool family today. Assembly is a workspace
(38 `assembly_*` tools), not a 3D-sketch mode.

Call **`cad_agent_guidance` first** and after every focus change. It
returns stage (`blank|sketch|solid|assembly|drawing`), counts, and the
next prompts/resources.

## How tutors should plug in

Do **not** put the exam compiler behind a single MCP tool. The exam is
the gold path; the MCP surface is the product.

| Surface | Role |
|---------|------|
| `prompts/get model_print_kit` | The recipe the agent (or a human) selects |
| `prompts/get tutor_exam` | How to *write* another exam |
| `nbcad://guidance` | Static packs + spine (cacheable) |
| `cad_agent_guidance` | Live next-steps from the current document |
| `scripts/fixtures/*.spec.json` | Numbers (`include_str` into Rust) |
| `print_kit_tutor.rs` + `mcp-print-kit-tutor.mjs` | Dual compilers, same calls |
| `nbcad-cli.mjs exam --stage=…` | Validate without reloading Cursor |

Integration order for a new tutor: write the prompt → keep the two
compilers in lockstep → grade in the harness → expose the prompt on
`prompts/list`. Resources are for **readable state**, not for dumping
the whole exam into context.

## What the print-kit taught us

Worked: named tools (not `cad_invoke` soup), role-based fits in the
spec, `solid_check` as a CLI, dual compilers, `server/discover` handshake,
soft disclosure once the right pack was visible.

Did not: Cursor often showed **0 tools** (`tools/list` lacked
`resultType`, `subscriptions/listen` was `-32601`, missing
`clientCapabilities` rejected modern calls). Agents were told to call
`cad_agent_guidance` but the tool did not exist. Default focus was
`document`, so `sketch_begin` and every `assembly_*` / motion /
`solid_check` tool were hidden until `cad_set_focus`. Waiting on
`notifications/initialized` timed out. Prompts said print-kit must not
form joints after we added them.
