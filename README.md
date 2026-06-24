# dod-org-tui

A cinematic terminal navigator for the DoD organizational structure — 149 nodes,
dual chain of command, wormhole jumps, and a live constellation minimap.

## Build

Requires Rust. [Install here](https://rustup.rs/).

```bash
cargo build --release
```

Binary: `target/release/dod-org-tui`

## Run

```bash
./target/release/dod-org-tui                        # auto-discovers the JSON
./target/release/dod-org-tui --data <path-to-json>  # explicit path
./target/release/dod-org-tui --no-anim              # static mode (SSH / battery)
```

## Controls

See [KEYMAP.md](KEYMAP.md) for the full reference. Quick start:

| Key | Action |
| --- | --- |
| `↑ / k`, `↓ / j` | move focus |
| `← / h`, `→ / l` | collapse / expand branch |
| `Space / Enter` | toggle expand/collapse |
| `D / U`, `PageDn / PageUp` | half-page / full-page scroll |
| `gg / Home`, `G / End` | jump to top / bottom |
| `{ / }` | previous / next sibling |
| `[ / ]` | previous / next node of the same type |
| `m{a-z}` / `'{a-z}` | set / jump to a named mark |
| `za / zM / zR` | toggle fold / collapse all / expand all |
| `/` | live search (type to filter; `n / N` to step matches) |
| `d` | toggle unit-data card (right panel) |
| `Tab` | cycle focus: tree → card → links → tree |
| `Enter` | (on link) open in browser |
| `o` | toggle ADMIN ⇄ OPS mode |
| `1`–`9` | wormhole-jump to the Nth OPS relation |
| `gd` | wormhole-jump along edge 1 |
| `q / Ctrl+C` | quit |

## Visual features

- **Constellation minimap** — radial star-field of all 149 nodes; breadcrumb path
  lit in blue; traveling spark pulses along your ancestry chain.
- **Type-coded colors** — gold for apex, blue for OSD/secretariat, violet for
  Joint, green for services, amber/crimson for COCOMs, teal for commands, slate
  for agencies.
- **Echelon shading** — apex nodes are brightest; depth dims toward the leaves.
- **Boot cascade** — banner flickers in, tree grows out from DoD (~1.6 s).
- **Breathing cursor** — focused node glows softly in the tree and pulses on the
  constellation.
- **OPS mode** — press `o` to switch to the operational chain. A relations rail
  lists numbered jump targets; arcs flow on the minimap, colored by relation type.
- **Wormhole jump** — `gd` or `1`–`9` launches an animated comet along a bezier
  arc to the target node; auto-expands ancestors and refocuses.
- **Classification banner** — top bar: classification badge + typewriter title on
  boot; badge shimmers gently while running.

## The data

Reads from `dod-org-data-research-ingest.json`. Auto-discovered from the current
directory, beside the executable, or two levels up (project root). Override with
`--data <path>`.
