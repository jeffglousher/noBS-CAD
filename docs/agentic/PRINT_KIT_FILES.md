# Print-kit files — which copy is live

Inventoried 2026-08-17 (local Documents + this branch).
Hashes are SHA-256 prefixes (16 hex). Replay the live kit with
`npm run test:mcp-print-kit` — that **overwrites**
`Documents\noBS-CAD\Print-Kit-Tutor\` and the sibling
`Print-Kit-Tutor.nbcad` / `-design.md` / `-report.json`.

## Live aero kit (print this)

**NACA 0024-4.5/3.5 · 8 rollers · 90.5 cm³ · 47.2 g · $0.94**
SHA-256 of the 3MF starts `BA828E74708C3801`. Size **437399** bytes.
Written **2026-08-17 18:42:34**.

| Path | What it is |
|------|------------|
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor\01-kit.3mf` | Exam export (same bytes as Latest) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Latest\01-kit.3mf` | Working alias of the same 3MF |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-01-kit-stator.3mf` | Loose root copy, **same hash** |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad` | Assembled project (`DEC1977D965E6D21`) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Latest\Print-Kit-Latest.nbcad` | Same `.nbcad` bytes |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-design.md` | Design report (`40B4F1ED87363F82`) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-report.json` | Grader JSON (`D260A2B1F7F60A09`) |
| `scripts/fixtures/print-kit-tutor.spec.json` | Source numbers (git) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Aero-Live-2026-08-17\` | **Dated backup** of the live set |

Close Bambu **without saving** over `Print-Kit-Tutor\01-kit.3mf` if an older plate is still open.

## Not the live kit

| Path | What it actually is | Hash / size |
|------|--------------------|-------------|
| `%USERPROFILE%\Documents\noBS-CAD\01-kit.3mf` | **Stale.** Morning organic **NACA 0021** / 6-roller kit | `EDD60FFA38A35B75` / 334073 / 11:35 |
| `Print-Kit-Organic-Reference\01-kit.3mf` | Same 0021 bytes — **appearance reference only** | same `EDD60F…` |
| `Print-Kit-Tutor-01-kit-under-root.3mf` | Older under-root experiment | `7D6D5809778E764A` / 55908 / 07:18 |
| `Print-Kit-Inner-Ring\` | Retrofit slip hoop for the **original printed stator** (race ID 48.5, no keepers). Not part of the new kit. | 3MF `F29EB91EB5E47138` / 8547 |
| `%USERPROFILE%\Documents\noBS-CAD\01-inner-ring.3mf` | Later / different ring export at the Documents root | `D89A0270B8D47116` / 41886 / 18:49 |

Organic look (root blend, tip landing) is frozen in
[PRINT_KIT_ORGANIC_REFERENCE.md](PRINT_KIT_ORGANIC_REFERENCE.md).
Aero contract and mathematics:
[PRINT_KIT_AERO.md](PRINT_KIT_AERO.md).
