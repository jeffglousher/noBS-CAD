# noBS CAD knowledge update log

## 2026-08-15

- **Update**: MCP main surface now advertises focused reads for sketch,
  profiles, features, visibility, appearances, materials, and workspace, plus
  recipes for import/export STEP, profile solids, drawing export, undo, and
  `cad_invoke`. HLR projection and session focus/window templates stay as
  tools, not resources.

## 2026-08-13

- **Update**: MCP harness concept now records recommended protocol
  `2026-07-28` (`server/discover` + `_meta`). `initialize` is a compatibility
  pathway only; the first legacy reply includes the runtime-upgrade manual.
  Live co-link takes a UI-published writer lock; UI heartbeat must not clobber
  MCP revisions (`npm run test:session-bridge`). Switching `cad_attach` to
  another session releases the previous live writer; a failed switch keeps it.
  MCP resources (`nbcad://…`) and prompts cover document/scene/drawing/sessions;
  drawing DTO + Browser visibility tools wrap existing engine methods.

## 2026-07-29

- **Update**: Aligned the bundle with OKF v0.2 and current `main`.
- **Validation**: Added automated structure and internal-link checks.

## 2026-07-28

- **Update**: Aligned concepts with maintainer feedback — goals vs proposals,
  co-link first / multi-window deferred, and agent steering files kept internal.

## 2026-07-27

- **Creation**: Seeded the bundle from the README product stance and MCP docs.
