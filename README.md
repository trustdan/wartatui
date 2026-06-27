<div align="center">

# wartatui

### A cinematic terminal navigator for the U.S. Department of Defense org chart

**149 nodes · dual chain of command · wormhole jumps · live constellation minimap**

[![Rust](https://img.shields.io/badge/built_with-Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![ratatui](https://img.shields.io/badge/TUI-ratatui-7c5cdf)](https://ratatui.rs/)
[![Classification](https://img.shields.io/badge/UNCLASSIFIED-00c800)](#the-data)
[![License](https://img.shields.io/badge/license-MIT-blue)](#license)

<br />

![wartatui in motion](https://raw.githubusercontent.com/trustdan/wartatui/main/wartatui.gif)

</div>

---

## Why this exists

The DoD has **two** org charts, and the gap between them is the whole story:

- **Administrative chain** — the Title 10 *"organize, train, and equip"* hierarchy. The chart everyone draws.
- **Operational chain** — who actually *provides forces to* whom, who *fights for* whom.

Navy Pacific Fleet belongs to the Navy on paper, but answers to INDOPACOM in a fight. That gap is the most confusing — and most interesting — thing about how the Pentagon works, and no web org chart shows it well. **`wartatui` makes it navigable.** Press `o`, and the operational edges light up across the tree.

---

## Features

- 🌌 **Constellation minimap** — a radial star-field of all 149 nodes; your breadcrumb path lit in blue, a traveling spark pulsing along your ancestry chain.
- 🪐 **Wormhole jumps** — `gd` or `1`–`9` launches an animated comet along a bezier arc to a node's operational partner, auto-expanding ancestors and refocusing.
- 🎚️ **Dual-mode** — toggle **ADMIN ⇄ OPS** with `o`. OPS overlays glowing braille edges, flowing source→target, colored by relation type.
- 🎨 **Type-coded colors** — hue encodes *what* a node is (gold apex, blue secretariat, violet Joint, green services, warm COCOMs, teal commands, slate agencies); brightness encodes *how deep* it sits.
- ⌨️ **Vim navigation** — `hjkl`, folds (`za`/`zM`/`zR`), marks (`m{a-z}` / `'{a-z}`), same-type cycling (`[` / `]`), live search (`/`).
- 🃏 **Unit cards** — every node carries its U.S.C. statutory citation, HQ, commander, and a source link you can open in your browser.
- ✨ **Full cinematic render loop** — boot cascade, breathing cursor glow, twinkle, shimmering classification banner. (`--no-anim` turns it all off for SSH / battery.)

---

## Install & run

Requires [Rust](https://rustup.rs/).

```bash
git clone https://github.com/trustdan/wartatui
cd wartatui
cargo build --release
```

```bash
./target/release/dod-org-tui                        # auto-discovers the JSON
./target/release/dod-org-tui --data <path-to-json>  # explicit data path
./target/release/dod-org-tui --no-anim              # static mode (SSH / battery)
```

---

## Quick keys

| Key | Action |
| --- | --- |
| `↑/k` `↓/j` | move focus |
| `←/h` `→/l` | collapse / expand branch |
| `/` | live search (`n` / `N` to step matches) |
| `[` `]` | previous / next node of the **same type** |
| `o` | toggle **ADMIN ⇄ OPS** mode |
| `gd` · `1`–`9` | **wormhole-jump** along an operational edge |
| `m{a-z}` `'{a-z}` | set / jump to a named mark |
| `d` | toggle the unit-data card |
| `Tab` | cycle focus: tree → card → links |
| `q` / `Ctrl+C` | quit |

Full reference in **[KEYMAP.md](KEYMAP.md)**.

---

## The data

Reads from `dod-org-data-research-ingest.json`: **149 nodes, 13 edges, 6 echelons**, rooted at the Department of Defense. Every node carries a U.S.C. citation, source URL, confidence, and notes. The file is auto-discovered from the current directory, beside the executable, or at the project root — override with `--data <path>`.

The banner classification is **data-driven** (`meta.classification`), not hard-coded — currently `UNCLASSIFIED`.

---

## Project docs

| Doc | What's in it |
| --- | --- |
| **[DESIGN.md](DESIGN.md)** | the soul of the app — dual chains, locked decisions, architecture |
| **[THEME.md](THEME.md)** | the color contract — hue per type, echelon shading, edge colors |
| **[KEYMAP.md](KEYMAP.md)** | every keybinding |
| **[ROADMAP.md](ROADMAP.md)** | build phases |

---

## License

MIT.
