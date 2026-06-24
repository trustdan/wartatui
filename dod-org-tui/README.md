# dod-org-tui — Ratatui Terminal UI

A fast, colorful terminal navigator for the DoD org structure.

## Build

Requires Rust. [Install here](https://rustup.rs/).

```bash
cd dod-org-tui
cargo build --release
```

Binary will be at `target/release/dod-org-tui`.

## Run

```bash
./target/release/dod-org-tui dod-org-data-research-ingest.json
```

Or with default path (JSON in repo root):

```bash
./target/release/dod-org-tui
```

## Controls

| Key               | Action                               |
| ----------------- | ------------------------------------ |
| **↑ / k**         | Move focus up                        |
| **↓ / j**         | Move focus down                      |
| **← / h**         | Collapse branch or jump to parent    |
| **→ / l**         | Expand branch or jump to first child |
| **Space / Enter** | Toggle expand/collapse               |
| **/**             | Enter search mode (type to filter)   |
| **d**             | Toggle unit-data card (right panel)  |
| **Esc**           | Close search or card                 |
| **q / Ctrl+C**    | Quit                                 |

## Features

- **Full tree navigation** with keyboard (vim-style hjkl, arrow keys)
- **Live search** with `/` — type a name, see matches + ancestors highlighted
- **Unit-data card** — press `d` to see echelon, type, reports-to, statutory authority
- **Org-type colors** — purple for Joint, gold for apex, blue for OSD, etc.
- **Auto-expand** — top echelons (0–2) expand by default so you see structure immediately

## The data

The TUI reads from `dod-org-data-research-ingest.json` in the project root.
