# dod-org-tui — Design

The single source of truth for what we're building and why. If an implementation
choice ever contradicts this doc, the doc wins (or the doc gets updated first).

## The soul: DoD has two org charts

The dataset (`dod-org-data-research-ingest.json`) encodes **both** chains of command:

- **Administrative chain** — the `parent` links between the 149 nodes. This is the
  Title 10 "organize, train, and equip" hierarchy. The chart everyone draws.
- **Operational chain** — the 13 `edges` with relation types. This is the real one:
  who actually provides forces to whom, who fights for whom.

| Relation | Count | Meaning |
| --- | --- | --- |
| `combat_support_agency_for` | 4 | DLA / DISA / DIA / DTRA → the warfight |
| `service_component_of` | 3 | e.g. AFSOC → USSOCOM |
| `provides_forces_to` | 3 | e.g. Navy Pacific Fleet → INDOPACOM |
| `oversight_by` | 2 | civilian/joint oversight |
| `reports_operationally_to` | 1 | operational reporting |

The gap between these two charts — Navy Pacific Fleet belongs to the Navy on paper
but answers to INDOPACOM in a fight — is the most confusing and most interesting
thing about how DoD works, and **no web org chart shows it well.** Making that gap
navigable is the whole point of this app.

## Locked decisions

- **Centerpiece:** the dual-chain. ADMIN tree ⇄ OPS edge overlay, with animated
  "wormhole" jumps along operational edges.
- **Signature visual:** a radial **constellation minimap** (Canvas/braille) of the
  whole org, current node pulsing, breadcrumb path lit.
- **Motion:** full cinematic — boot sequence, ~30–60fps render loop, breathing
  cursor glow, flowing edges, twinkle.

## The data (validated against the model)

- 149 nodes, 13 edges, 6 echelons (E0–E5), 1 root (`department-of-defense`).
- 17 node `type`s (see [THEME.md](THEME.md) for the color families).
- Every node has `source` (a U.S.C. statutory citation) and `meta` with
  `sourceUrl`, `confidence`, `notes` (often HQ + commander), and sometimes
  `displayAlias`. These feed the unit card.

## Modes

- **ADMIN** (default) — the parent tree, vim-navigable.
- **OPS** (`o`) — operational edges light up; on a node with relations, glowing
  braille lines flow to its partners (flow direction = relation direction), and a
  right-rail lists them as numbered jump targets.

## The wormhole jump

Land on a node with an operational edge, press `gd` (or pick a numbered relation),
and the view leaps across the edge to the target — the tree reorients and the
constellation arcs to bring it into focus, over a ~0.5s eased transition. This is
how you "zip between parallel, tangential, cross-cutting agencies."

## Screen layout

```
┌ UNCLASSIFIED · DoD Organizational Structure · asOf 2026-06-23 ┐
├ TREE (admin) ~55% ───────────┬ UNIT CARD / RELATIONS ~45% ────┤
│ ▼ Department of Defense       │ USSOCOM                        │
│  ▼ Secretary of Defense       │ U.S. Special Operations Cmd    │
│   ◉ USSOCOM ◄                 │ 10 U.S.C. §161 · E2 · func     │
│    ...                        │ ⟶ RELATIONS (OPS)              │
│                               │  1▸ AFSOC  service_component_of│
├ CONSTELLATION ── radial star-map, node pulsing, path + edges ──┤
├ ADMIN · DoD › SecDef › USSOCOM · [o]ps /search gd→jump ────────┤
```

## Architecture

Full cinematic means a **frame clock**, not poll-and-redraw: render at a fixed
cadence driven by one global `elapsed` time; input is just another event that
nudges state. All animation (boot cascade, glow, flowing edges, wormhole) reads
from that clock plus an eased transition state.

Module layout:

```
src/
  main.rs          terminal setup + the frame loop
  model.rs         data structs + JSON loading
  app.rs           App state, modes (ADMIN/OPS), input → state
  tree.rs          tree state, flatten, vim navigation
  theme.rs         RGB palette per type, echelon brightness shading
  anim.rs          clock, easing, transition/boot timelines
  layout_radial.rs node → (x,y) radial positions for the constellation
  ui/
    banner.rs      classification banner (shimmer)
    tree_view.rs   the admin tree
    card.rs        unit card + relations rail
    constellation.rs  the Canvas star-map
    statusline.rs  mode + breadcrumb + key hints
```

The constellation is a radial/sunburst layout: DoD at center, each echelon a ring
outward (radius ∝ echelon, angle = slot within echelon). Deterministic, reads like
a star map.

See [ROADMAP.md](ROADMAP.md) for the build phases and [KEYMAP.md](KEYMAP.md) for
the full keybindings.
