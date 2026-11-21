# Construction Sticks for Horizon

A port of **Construction Sticks 1.5.0** ([Mrbysco/ConstructionSticks](https://github.com/Mrbysco/ConstructionSticks),
MIT) for **Minecraft Java 26.3 on Horizon**, built against the Horizon mod SDK
(`api = "0.2"`, nightly `0a875da`).

One flat `.hmod` carries both halves: the five construction sticks as carried content
(`assets/`, `data/`) and the behaviour as native code (`src/`).

## What this slice implements

- Five sticks — wooden, copper, iron, diamond, netherite — with the tier limits
  **3 / 9 / 27 / 128 / 1024** blocks per use and the tier durabilities **59 / 131 / 250
  / 1561 / 2031** (one per stack, as upstream).
- A right click with a stick on a block fills the **plane of the clicked face** with
  copies of the clicked block: the fill starts at `clicked + face` and grows over both
  in-plane axes and their diagonals, accepting a cell only when the block behind it is
  still the clicked block.
- The fill is capped by the tier limit, the stick's **remaining durability** and, outside
  creative, the supply. Each placed block consumes one item of the clicked block's kind
  and one durability; a creative player places for free and never wears the stick. If the
  supply runs out mid-run, the last block is rolled back; a stick that reaches its
  maximum damage breaks on that use.

## Not ported yet

The angel, destruction and replacement modes, the five upgrade templates (and the recipe
that applies them), the battery, the options screen, the keybinds, the undo history,
container-sourced supply and the client preview. The carried content includes their
items, recipes and models; the behaviour is not there. The gaps are listed in
[`notes.md`](notes.md#gaps-020).

## Layout

```
horizon-mod.toml   the manifest: identity, api 0.2, side, capabilities, [native] pins
Cargo.toml         one crate; the only dependency is horizon-mod-api from .horizon/sdk/
src/lib.rs         the one impl mod_api::Hooks (attaches the stick Part)
src/support.rs     the glue: the clicked state, the supply, the placement, the wear
src/runtime.rs     the pure planner, its limits and their tests
assets/, data/     the carried content: models, textures, lang, recipes, tags
```

## Build

```sh
cargo test --offline                 # the pure planner, its limits and the glue helpers
cargo clippy --all-targets -- -D warnings
cargo fmt --check
rm -f Cargo.lock && cargo horizon-mod build      # -> target/horizon-mod/constructionstick.hmod
cargo horizon-mod check constructionstick.hmod
cargo horizon-mod fits constructionstick.hmod --build nightly
```

Any `cargo horizon-mod` command fetches the pinned SDK into `.horizon/sdk/`
(git-ignored) on first use; the launcher builds the native half against its own copy of
the same build. The in-game check is `tools/e2e.sh`, which needs a **launcher-built**
game (`GAME=<horizon-client> horizon-launcher native-build`).

## Licence

MIT. The original copyright notice travels in `LICENSE` and `LICENSE-NOTICE.md`, both
inside the package. Local-only until the maintainer approves publishing.
