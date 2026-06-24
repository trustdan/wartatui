# dod-org-tui — Roadmap

This is the control surface. Each phase is a branch + commit. A phase is **done**
when every box in its Definition of Done is ticked, you've run the binary and
signed off, and it's merged to `main`. Then you say "next."

## The per-phase ritual

1. I build the phase on a branch (`phase-N-<name>`).
2. It compiles (`cargo build`) with no errors.
3. **You run it** (`cargo run --release`) and watch it.
4. We check it against the Definition of Done below.
5. 👍 → commit + merge to `main` → you say "next."

Estimates are in *build sessions* (a focused build→run→react sitting), not
engineering-weeks. The real clock is review + taste iteration, not my typing.

---

## Phase 1 · Foundation  ·  ✅ DONE (merged 2026-06-24)

Module split, the frame-clock render loop, the color system, and the boot cascade.
After this it already moves.

- [x] `main.rs` split into the modules in [DESIGN.md](DESIGN.md)
- [x] Frame-clock loop renders at a steady ~30fps with a global `elapsed` clock
- [x] Input decoupled from render (no missed/laggy keys)
- [x] RGB theme applied: each of 17 types colored per [THEME.md](THEME.md)
- [x] Echelon brightness shading visibly conveys depth
- [x] Boot sequence: banner flickers in, tree grows out from DoD (~1.5s)
- [x] Breathing cursor glow on the focused node
- [x] Tree + unit card render cleanly; card shows fullName, type, echelon,
      statutory authority, HQ/commander from `notes`, confidence
- [x] Existing nav still works (hjkl/arrows, space/enter, `/` search, `d`, `q`)
- [x] Bonus: robust `--data` flag + data-file auto-discovery

## Phase 2 · Constellation  ·  ✅ DONE (merged 2026-06-24)

The signature visual.

- [x] `layout_radial.rs` computes deterministic positions for all 149 nodes
- [x] Canvas minimap renders the full org as a star-field
- [x] Current node pulses; breadcrumb path to root is lit
- [x] Minimap stays readable at small terminal sizes (graceful degrade)
- [x] Selecting nodes in the tree moves the pulse smoothly in the minimap
- [x] Bonus: panel focus + scrollable card w/ scrollbar; vim paging
      (Shift+D/U, PageUp/Dn, gg/G); in-card link selection + browser open;
      ellipse fill (no clipping); "you are here" corner orientation label

## Phase 3 · Dual-chain  ·  ~1–1.5 sessions

The centerpiece.

- [ ] `o` toggles ADMIN ⇄ OPS; mode shown in status line
- [ ] OPS: relations rail lists this node's edges as numbered jump targets
- [ ] OPS: edges draw on the constellation as arcs, colored by relation
- [ ] Flowing-dash animation along edges; flow direction = relation direction
- [ ] `gd` (and number keys) perform the wormhole jump with an eased transition
- [ ] Jump auto-expands the target's ancestors and refocuses correctly

## Phase 4 · Vim power-moves  ·  ~0.5–1 session

- [ ] `gg` / `G` top / bottom
- [ ] `{` / `}` prev / next sibling
- [ ] `[` / `]` prev / next node of the same `type` (cross-cutting)
- [ ] `m{a-z}` set mark, `'{a-z}` jump to mark
- [ ] `zM` / `zR` / `za` collapse all / expand all / toggle fold
- [ ] `n` / `N` next / prev search match
- [ ] All bindings match [KEYMAP.md](KEYMAP.md) exactly

## Phase 5 · Polish  ·  ~0.5–1+ session (open-ended)

- [ ] Easing pass on every transition (no linear/janky motion)
- [ ] Color tuning for contrast + vibrancy across terminals
- [ ] Classification banner shimmer
- [ ] Perf check: steady frame rate, low idle CPU
- [ ] README + KEYMAP updated to final behavior
- [ ] Optional: `--no-anim` / motion toggle for SSH/battery

---

## Decision log

- 2026-06-24 — Centerpiece = dual-chain; signature = constellation; motion = full
  cinematic. (See [DESIGN.md](DESIGN.md).)

## Open questions

- *(none yet — add as they come up)*
