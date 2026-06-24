# dod-org-tui — Keymap

Vim-style. Designed so you stop thinking in the tree and start thinking in
relationships.

## Movement

| Key | Action |
| --- | --- |
| `↓` / `j` | move focus down |
| `↑` / `k` | move focus up |
| `→` / `l` | expand branch, or jump to first child |
| `←` / `h` | collapse branch / jump to parent / leave card |
| `D` / `U` | half-page down / up (Shift+d / Shift+u) |
| `PageDn` / `PageUp` | full-page down / up |
| `gg` / `Home` | jump to top |
| `G` / `End` | jump to bottom |
| `}` | next sibling |
| `{` | previous sibling |

Movement keys act on whichever panel has focus: the **tree** moves the
selection; the **card** scrolls its contents.

## Panels & links

| Key | Action |
| --- | --- |
| `Tab` | Tree → Card → cycle each link → back to Tree |
| `Shift+Tab` | jump focus straight back to the tree |
| `h` / `←` | (from card) return focus to the tree |
| `Enter` | (on a selected link) open it in the browser |

## Cross-cutting (the point of the app)

| Key | Action |
| --- | --- |
| `]` | next node of the **same type** (e.g. cycle all geo-COCOMs) |
| `[` | previous node of the same type |
| `gd` | **wormhole jump** along this node's first operational edge |
| `1`–`9` | jump to the Nth listed relation (OPS mode) |
| `m{a-z}` | set a named mark on the current node |
| `'{a-z}` | jump back to a mark (bounce between two agencies) |

## Folding

| Key | Action |
| --- | --- |
| `Space` / `Enter` | toggle expand/collapse on focus |
| `za` | toggle fold |
| `zM` | collapse all |
| `zR` | expand all |

## Modes, search & misc

| Key | Action |
| --- | --- |
| `o` | toggle ADMIN ⇄ OPS mode |
| `d` | toggle the unit-data card |
| `/` | enter search (type to filter visible nodes) |
| `n` / `N` | next / previous search match |
| `Esc` | close search / card / cancel pending prefix |
| `q` / `Ctrl+C` | quit |

## Notes

- Multi-key sequences (`gg`, `gd`, `m{x}`, `'{x}`, `z*`) use a short
  pending-key window; an unmatched second key cancels the sequence.
- Number keys `1`–`9` only act as relation-jumps in OPS mode when a relations
  list is visible; otherwise they're ignored.
- `--no-anim` (CLI flag) disables all motion — useful over SSH or on battery.
  Idle CPU drops from ~30 fps polling to ~10 fps.
