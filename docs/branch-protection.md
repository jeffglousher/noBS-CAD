# Branch protection checklist (maintainers)

## Current state

- Pull requests to `main` should be reviewed.
- **One approving review** is the expected bar (either `@jackControls` or
  `@jeffglousher` may approve the other’s PR; authors do not self-approve).
- **Required CI status checks** are still **deferred** until reliable jobs
  exist (see engine/MCP CI work). The desktop packaging workflow is valuable
  but is not treated as a hard required gate for every docs PR yet.

## When enabling a formal `main` protection rule

GitHub → Settings → Branches → Branch protection rule for `main`:

- [ ] Require a pull request before merging
- [ ] Require approvals: 1
- [ ] Dismiss stale approvals when new commits are pushed
- [ ] Require review from Code Owners (once handles stay accurate)
- [ ] Require status checks to pass — **only after** named jobs are stable
- [ ] Require branches to be up to date before merging (optional; cost vs safety)
- [ ] Do not allow force pushes
- [ ] Do not allow deletions

## Suggested required checks (later)

| Check | Purpose | Required? |
|-------|---------|-----------|
| `Engine / cargo-test` | `cargo test --workspace` (`.github/workflows/ci-engine.yml`) | When green and fast enough |
| `Browser / WASM / browser-smoke` | wasm build + `smoke:wasm` + `smoke:colink` (`.github/workflows/ci-browser.yml`) | Soft → then required |
| `MCP server / mcp-tests` | Windows OCCT + mcp-server tests | When green |
| Windows portable ZIP | existing desktop packaging workflow | Informative until stable/fast for all PRs |

## Follow-through

PRs that fail checks or accumulate unresolved review threads should be followed
through to merge-ready rather than abandoned. See `CONTRIBUTING.md`.
