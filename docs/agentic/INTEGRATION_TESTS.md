# MCP integration tests and pipeline goldens

This is the ordered list of **MCP + kernel goldens**. Start here when
proving the server can replay a real modeling pipeline — not only that
tools exist.

**#1 is the print-kit pipeline.** It is a deterministic command sequence
that builds an FDM-tolerant mechanical **assembly**. The same spec and
the same tool order produce the same kit. That is useful context for an
agent. It is not a test of AI capability.

Curriculum and runner: [PRINT_KIT_TUTOR.md](PRINT_KIT_TUTOR.md).
Spec: `scripts/fixtures/print-kit-tutor.spec.json`. Recipe: `model_print_kit`.

## Ordered goldens

| # | Golden | How to run | What it proves |
|---|--------|------------|----------------|
| **1** | **Print-kit pipeline** (printed VAWT assembly) | `cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor` then `npm run test:mcp-print-kit` | MCP tools replay a kit: one-piece helical NACA, role-based fits, hollow PETG roller pack, assembly drawing, design report + plastic cost, one laid-out plate (PLA Orange + PLA Glow + PETG HF) |
| 2 | Completeness gate | `npm run check:mcp-control` | Modeling / print / control tools and main prompts stay wired (`model_print_kit` included) |
| 3 | CadServer goldens | `cargo test --manifest-path mcp-server/Cargo.toml` | Headless OCCT replay and MCP RPC (includes the #1 engine exam) |
| 4 | Session bridge | `npm run test:session-bridge` | Live attach, writer lock, UI heartbeat must not clobber MCP revisions |

Windows needs OCCT on `PATH` for #1 and #3:

```powershell
$env:OCCT_ROOT = "$PWD\vcpkg_installed\x64-windows"
$env:Path = "$env:OCCT_ROOT\bin;$env:Path"
```

CI (`.github/workflows/mcp-server.yml`) runs #2, then #3, then the Node half
of #1. Optional live desktop for #1: `node scripts/mcp-print-kit-tutor.mjs --live`.
The Node exam also writes `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad`
and one laid-out `01-kit.3mf` under `Print-Kit-Tutor\` (override with
`NBCAD_PROJECT_OUT` / `NBCAD_3MF_DIR`). The exam deletes retired plates
from earlier kits before writing. The roller cartridge is PIP.
Do not print the assembled nest.

Supporting crate jobs (export, xtask `install-mcp`) are packaging checks, not
modeling pipelines. Do not insert them above #1.

## Why #1 is first

Tool-count and RPC goldens can pass without ever building a part. The
print-kit pipeline is the first full replay: solids, assembly, drawing,
and one laid-out 3MF ([ASSEMBLY.md](ASSEMBLY.md)). Later catalog bearings
(608 and up from a standard table) are optional hardware, not part of
this kit.
