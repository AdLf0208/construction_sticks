# Construction Sticks — a Horizon port

**Construction Sticks** is a Minecraft mod. Sticks that make construction easier. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [Mrbysco/ConstructionSticks](https://github.com/Mrbysco/ConstructionSticks) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Construction Sticks ([source](https://github.com/Mrbysco/ConstructionSticks)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `constructionstick.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What this slice implements

- Five sticks — wooden, copper, iron, diamond, netherite — with the tier limits
  **3 / 9 / 27 / 128 / 1024** blocks per use.
- A right click with a stick on a block fills the **plane of the clicked face** with
  copies of the clicked block: the fill starts at `clicked + face` and grows over both
  in-plane axes and their diagonals, accepting a cell only when the block behind it is
  still the clicked block.
- The fill is capped by `min(tier limit, supply)`. Each placed block consumes one item
  of the clicked block's kind and wears the stick by one durability; a creative player
  places for free. If the supply runs out mid-run, the last block is rolled back.

## Not yet ported

- Placement uses `default_state` of the clicked block (item id = block id); a
- Only `minecraft:air` counts as placeable (upstream also replaces replaceable
- No options screen/undo/keys, no angel/destruction/replacement modes, no
- No `native_play_sound` call yet (one place sound per job upstream), no place
- **No entity obstruction.** `level.entities_in(level, min, max)` returns `EntityRef`s and
- **No `canSurvive` query** (the same gap `buildinggadgets2` recorded): a state that cannot
- **No `getStateForPlacement`**: the clicked state is copied instead (above).
- **No place sound, game event, statistic or `placed_block` advancement trigger**, and

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `constructionstick.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Construction Sticks was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/Mrbysco/ConstructionSticks

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run constructionstick` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/constructionstick/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
