# dod-org-tui — Theme

The color contract. Hue encodes **what kind of thing** a node is; brightness
encodes **how deep** it sits (echelon). Together they let you read the org at a
glance. RGB values are starting points — Phase 5 tunes them for contrast.

## Color families (by node `type`)

Grouped so related entities share a hue. Counts are from the dataset (149 nodes).

| Family | Types (count) | Base RGB | Reads as |
| --- | --- | --- | --- |
| **Apex / gold** | department (1), principal (2) | `255,205,70` | leadership |
| **Secretariat / blue** | osd (17) | `91,138,192` | civilian staff |
| | assistant-secretary (19) | `100,160,220` | |
| | deputy-assistant-secretary (17) | `140,185,230` | |
| **Joint / violet** | joint (5) | `155,127,224` | jointness |
| **Service / green** | mildep (3) | `80,165,105` | departments |
| | service (5) | `150,205,140` | armed services |
| **COCOM / warm** | cocom-geo (6) | `232,140,60` | geographic (orange) |
| | cocom-func (5) | `200,90,70` | functional (rust) |
| **Force-provider / teal** | major-command (18) | `55,180,175` | commands |
| | command (8) | `90,205,210` | |
| **Enterprise / slate** | agency (20) | `135,150,170` | defense agencies |
| | field-activity (7) | `110,160,170` | |
| | center (4) | `120,175,160` | |
| | lab (2) | `130,185,195` | |
| | directorate (10) | `150,162,178` | |

Anything unmatched → neutral gray `120,120,120`.

## Echelon shading

Echelon (E0 apex … E5 deepest) modulates lightness while keeping hue. Lower
echelons (closer to the apex) render brighter; deeper ones dim slightly so depth
is legible without changing what type a node is.

```
factor = lerp(1.15, 0.70, echelon / 5)   // E0 ≈ 1.15× brightness, E5 ≈ 0.70×
shaded = clamp(base_rgb * factor, 0..255)
```

The focused node overrides this with a breathing glow (a brightness pulse driven
by the animation clock) plus a dark-gray background highlight.

## Edge colors (OPS mode)

Operational edges are colored by relation so the chart "reads" even before you read
the labels.

| Relation | RGB | Mnemonic |
| --- | --- | --- |
| `provides_forces_to` | `232,140,60` | force flow = warm orange |
| `service_component_of` | `90,205,210` | component = cyan |
| `combat_support_agency_for` | `120,200,140` | support = green |
| `oversight_by` | `175,150,230` | oversight = violet |
| `reports_operationally_to` | `235,205,90` | reporting = gold |

Flowing dashes animate **from source → target** so direction is unambiguous.

## Classification banner

Banner reads `UNCLASSIFIED` in green (`0,200,0`) per the dataset's
`meta.classification`. If a future dataset declares a higher classification, the
banner color/label must change accordingly — treat it as data-driven, never
hard-coded to green.
