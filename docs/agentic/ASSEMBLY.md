# Assembly gap

noBS CAD can keep **several solid bodies in one document**. That is a
multi-body part, not an assembly.

## What exists

- `solid_*` `operation: "new_body"` creates independent bodies
- Browser **Bodies** folder, per-body appearance / 3MF materials
- Combine, split, mirror, rectangular / circular pattern
- Manual placement by sketching on datums (no `solid_move`)

## What does not exist

| Missing | Why it matters |
|---------|----------------|
| Part instances / occurrence tree | You cannot reuse one part file in many places |
| Mates / joints | Fits are numbers in sketches, not constraints |
| Assembled vs exploded vs print layouts | One placement only; the tutor builds the **assembled stack** by construction |
| Catalog hardware | A metal 608 (or any standard bearing) is not a first-class component. Larger / better bearings can be modeled later from a table — they are not hidden parts of the print-kit exam |
| Kinematics | `docs/goals.md` lists motion as needing assemblies and joints |

The print-kit tutor (benchmark #1) is honest about this: it places six
printable bodies on one axis so a human can see how they go together. That
placement is brittle on purpose. Closing the gap is a product project, not a
prompt tweak.

## Fully printable first

Until assemblies and catalog hardware exist, kits must **print every
bearing surface** they need (cone thrust, sleeve bushing, printed rollers).
Do not design a mechanism that only works if a metal bearing appears off
camera.
