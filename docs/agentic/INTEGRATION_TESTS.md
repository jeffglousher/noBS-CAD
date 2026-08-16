# MCP integration tests and benchmarks

This is the ordered list of **agentic CAD benchmarks**. Start here when
proving the MCP server can teach and grade real modeling — not only that
tools exist.

**#1 is the print-kit tutor.** It is the first synthesis exam: an agent
must design an FDM-tolerant mechanical kit, then a grader checks clearance,
orientation, machine elements, a mounted spinning part, and a 3MF package.

Curriculum and grader detail: [PRINT_KIT_TUTOR.md](PRINT_KIT_TUTOR.md).
GD&T / printability corrections: [PRINT_KIT_GDT.md](PRINT_KIT_GDT.md).
Spec: `scripts/fixtures/print-kit-tutor.spec.json`. Recipe: `model_print_kit`.

## Ordered benchmarks

| # | Benchmark | How to run | What it proves |
|---|-----------|------------|----------------|
| **1** | **Print-kit tutor** (printed turntable) | `cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor` then `npm run test:mcp-print-kit` | AI → assemblable printed product: coaxial stack, 45° cone thrust, printed sleeve, even 3-well platter, small keeper, 3MF |
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
and a matching 3MF (override with `NBCAD_PROJECT_OUT` / `NBCAD_3MF_OUT`).

Supporting crate jobs (export, xtask `install-mcp`) are packaging checks, not
synthesis benchmarks. Do not insert them above #1.

## Why #1 is first

Tool-count and RPC goldens can pass while an agent still emits a print-bed
scatter that will not assemble or spin. The print-kit tutor fails that
class of answer. It also records the product gap: we have multi-body parts,
not assemblies ([ASSEMBLY.md](ASSEMBLY.md)). Later catalog bearings (608
and up from a standard table) are optional hardware, not a hidden part of
this exam.
