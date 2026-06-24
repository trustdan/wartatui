# dod-org-tui — Keymap

The full keybinding contract. Vim-style, designed so you stop thinking in the tree
and start thinking in relationships. Phase numbers note when each binding lands.

## Movement

| Key | Action | Phase |
| --- | --- | --- |
| `↓` / `j` | move focus down | 1 |
| `↑` / `k` | move focus up | 1 |
| `→` / `l` | expand branch, or jump to first child | 1 |
| `←` / `h` | collapse branch, or jump to parent | 1 |
| `gg` | jump to root (top) | 4 |
| `G` | jump to last visible node | 4 |
| `}` | next sibling | 4 |
| `{` | previous sibling | 4 |

## Cross-cutting (the point of the app)

| Key | Action | Phase |
| --- | --- | --- |
| `]` | next node of the **same type** (e.g. cycle all geo-COCOMs) | 4 |
| `[` | previous node of the same type | 4 |
| `gd` | **wormhole jump** along this node's operational edge | 3 |
| `1`–`9` | jump to the Nth listed relation (OPS mode) | 3 |
| `m` `{a-z}` | set a mark on the current node | 4 |
| `'` `{a-z}` | jump back to a mark (bounce between two agencies) | 4 |

## Folding

| Key | Action | Phase |
| --- | --- | --- |
| `Space` / `Enter` | toggle expand/collapse on focus | 1 |
| `za` | toggle fold (vim alias) | 4 |
| `zM` | collapse all | 4 |
| `zR` | expand all | 4 |

## Modes & panels

| Key | Action | Phase |
| --- | --- | --- |
| `o` | toggle ADMIN ⇄ OPS mode | 3 |
| `d` | toggle the unit-data card | 1 |
| `/` | enter search | 1 |
| `n` / `N` | next / previous search match | 4 |
| `Esc` | close search / card / cancel | 1 |
| `q` / `Ctrl+C` | quit | 1 |

## Notes

- Multi-key sequences (`gg`, `gd`, `m{x}`, `'{x}`, `z*`) use a short pending-key
  timeout; an unmatched second key cancels the sequence.
- Number keys only act as relation-jumps while OPS mode shows a relations list;
  otherwise they're ignored.
