# Docs index

## Product / directions

| Doc | Audience | When to update |
|-----|----------|----------------|
| [goals.md](goals.md) | Everyone | Product direction changes |
| [proposed-architecture.md](proposed-architecture.md) | Everyone | When proposals ship or change |

## MCP / automation

| Doc | Audience | When to update |
|-----|----------|----------------|
| [mcp-harness.md](mcp-harness.md) | Humans + agents | Any MCP behavior change |
| [mcp-message-bus.md](mcp-message-bus.md) | System integrators | Kafka/MQTT/NATS bus envelope |
| [agent-mcp.md](agent-mcp.md) | Day-to-day agents | Workflow changes |
| [../mcp-server/README.md](../mcp-server/README.md) | Integrators | Tool surface / build |
| [../mcp-server/OKRs.md](../mcp-server/OKRs.md) | Maintainers | MCP milestone changes |

## Agentic / maintenance (committed project guidance)

| Doc | Audience | When to update |
|-----|----------|----------------|
| [agentic/INDEX.md](agentic/INDEX.md) | Agents + maintainers | Structure changes |
| [agentic/INTEGRATION_TESTS.md](agentic/INTEGRATION_TESTS.md) | Agents + CI | Ordered MCP benchmarks (#1 print-kit tutor) |
| [agentic/PRINT_KIT_TUTOR.md](agentic/PRINT_KIT_TUTOR.md) | Agents + CI | Curriculum / grader for benchmark #1 |
| [agentic/PRINT_KIT_GDT.md](agentic/PRINT_KIT_GDT.md) | Agents + CI | GD&T / printability study and corrections |
| [agentic/ASSEMBLY.md](agentic/ASSEMBLY.md) | Agents + maintainers | Multi-body vs assemblies gap |
| [agentic/STEERABLE_MCP.md](agentic/STEERABLE_MCP.md) | Agents | Disclosure invariants |
| [agentic/MCP_2026.md](agentic/MCP_2026.md) | Agents | Recommended MCP 2026-07-28 success manual |
| [agentic/MAINTENANCE.md](agentic/MAINTENANCE.md) | Agents + CI | Toolchain / test commands |
| [agentic/UI_OVERLAYS.md](agentic/UI_OVERLAYS.md) | UI agents + maintainers | Overlay or shell-layout changes |

## Manufacturing export

| Doc | Notes |
|-----|-------|
| [manufacturing/INDEX.md](manufacturing/INDEX.md) | 3MF/STL export subsystem |
| [manufacturing/DRAFT_PR.md](manufacturing/DRAFT_PR.md) | A+ PR summary (humans) |

## Packaging / provenance

| Doc | Notes |
|-----|-------|
| [OCCT_PACKAGING.md](OCCT_PACKAGING.md) | Native OCCT |
| [WINDOWS_PACKAGING.md](WINDOWS_PACKAGING.md) | Portable ZIP |
| [WINDOWS_NATIVE_VIEWPORT_DEBUGGING.md](WINDOWS_NATIVE_VIEWPORT_DEBUGGING.md) | Windows Bevy/WebView2 field-debugging runbook |
| [ICON_PROVENANCE.md](ICON_PROVENANCE.md) | Icons |

> Editor-specific files (`AGENTS.md`, `.cursor/`) stay **gitignored** per project policy.
> Use `docs/agentic/` for shared agent guidance that belongs in the repo.
