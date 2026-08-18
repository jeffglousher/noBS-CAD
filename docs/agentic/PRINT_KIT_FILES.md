# Print-kit files — which copy is live

Inventoried 2026-08-17 evening (bearing CAD + this branch).
Hashes are SHA-256 prefixes (16 hex). Replay the live kit with
`npm run test:mcp-print-kit` or `node scripts/nbcad-cli.mjs exam --stage=kit`
— that **overwrites** `Documents\noBS-CAD\Print-Kit-Tutor\` and the sibling
`Print-Kit-Tutor.nbcad` / `-design.md` / `-report.json`.

## Live bearing kit (print this)

**NACA 0024-4.5/3.5 · 8 hollow barrel-crowned PETG rollers · U-window fence · PETG E-clip · PLA Glow blades · 90.2 cm³ · 47 g · $0.94**

Hashes below are the **pre-PETG** solid-PLA crowned kit. Replay
`--stage=kit` after this pass and refresh this page.
SHA-256 of the 3MF starts `465883997473A724`. Size **472474** bytes.
Written **2026-08-17 19:56:20**.

| Path | What it is |
|------|------------|
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor\01-kit.3mf` | Exam export (same bytes as Latest) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Latest\01-kit.3mf` | Working alias of the same 3MF |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-01-kit-stator.3mf` | Loose root copy, **same hash** |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad` | Assembled project (`97CD3DFAA84FE50D`) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-design.md` | Design report (`ADD696C8E4D783CA`) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-report.json` | Grader JSON (`84516A2FB25F142E`) |
| `scripts/fixtures/print-kit-tutor.spec.json` | Source numbers (git) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Bearing-Live-2026-08-17\` | **Dated backup** of this live set |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Aero-Live-2026-08-17\` | Previous live aero (cylinder slots / C-washer), SHA `BA828E74708C3801` / 437399 |

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
Bearing CAD (U-window, crown, E-clip):
[PRINT_KIT_BEARING.md](PRINT_KIT_BEARING.md).
