# Print-kit files — which copy is live

Inventoried 2026-08-17 evening (PETG hollow-roller pass on this branch).
Hashes are SHA-256 prefixes (16 hex). Replay the live kit with
`npm run test:mcp-print-kit` or `node scripts/nbcad-cli.mjs exam --stage=kit`
— that **overwrites** `Documents\noBS-CAD\Print-Kit-Tutor\` and the sibling
`Print-Kit-Tutor.nbcad` / `-design.md` / `-report.json`.

## Live PETG kit (print this)

**NACA 0024-4.5/3.5 · 8 hollow barrel-crowned PETG rollers · U-window fence · PETG E-clip · PLA Glow blades · 89.8 cm³ · 46.8 g · $0.94**
SHA-256 of the 3MF starts `1AEB6B8F97A905ED`. Size **481592** bytes.
Written **2026-08-17 20:26:59**. READY TO PRINT (JS exam, 613 s).

| Path | What it is |
|------|------------|
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor\01-kit.3mf` | Exam export (same bytes as Latest) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Latest\01-kit.3mf` | Working alias of the same 3MF |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-01-kit-stator.3mf` | Loose root copy, **same hash** |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad` | Assembled project (`5B42149B8A02844E`) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-design.md` | Design report (`7AD0DB94CA552FDB`) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-report.json` | Grader JSON (`2A868FB4E36444B8`) |
| `scripts/fixtures/print-kit-tutor.spec.json` | Source numbers (git) |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Petg-Live-2026-08-17\` | **Dated backup** of this live set |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Bearing-Live-2026-08-17\` | Previous live bearing (solid PLA crown), SHA `465883997473A724` / 472474 |
| `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Aero-Live-2026-08-17\` | Previous live aero (cylinder slots / C-washer), SHA `BA828E74708C3801` / 437399 |

Slicer: three materials. Stator **PLA Basic Orange**. Rotor / blades **PLA Glow Green** (keep light for a later generator). Rollers + E-clip **PETG HF Black**. Dry PETG. Hardened nozzle. Close Bambu **without saving** over `Print-Kit-Tutor\01-kit.3mf` if an older plate is still open.

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
Bearing CAD + PETG pairing:
[PRINT_KIT_BEARING.md](PRINT_KIT_BEARING.md).
