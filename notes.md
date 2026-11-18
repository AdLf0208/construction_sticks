# Construction Sticks — port notes (P3 logic tier, fifth port)

Ported from **Construction Sticks 1.5.0** (`Mrbysco/ConstructionSticks`, MIT) for
Minecraft Java 1.21.1 NeoForge. Local-only port.

Status: `ported-partial` — **slice 1 (construction)** implemented: five sticks,
plane fill of the clicked face, support matching, tier limit and item
consumption, worn stick. Angel/destruction/replacement modes, upgrades, the
options screen/undo and the battery are later slices.

> **0.2.0 (2026-11-18):** the package moved from the retired `.hnmod` container to
> the flat SDK `.hmod` (`api = "0.2"`, one `impl mod_api::Hooks`, content at the
> root). Everything below the migration section is the 0.1.0 record; the file map,
> the helper-to-hook table, the current gaps and the verification are under
> [`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18).


## Behaviour implemented (slice 1)

- A right click with a stick on a block claims the click and fills the **plane
  of the clicked face** with copies of the clicked block: start at
  `clicked + face.normal`, breadth-first over both in-plane axes and their
  diagonals, accepting a cell only when the block behind it (`cell - normal`)
  is still the clicked block.
- Bounded by the tier limit (`3/9/27/128/1024`) and the player's supply; each
  placed cell consumes one item of the clicked block's kind and wears the held
  stick by one; if the supply runs out mid-run the last block is rolled back.
- Creative places without consuming (as upstream).

## Hooks

*(The 0.1.0 hooks, for the record. The current SDK method and the host calls that
replaced each old `Game::` helper are in
[`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18).)*

| Hook | Helper | What it does |
|---|---|---|
| `use_item_on` (v1, HEAD) | `use_on.rs` | On a stick: read the clicked block and the plane, claim the use, place and consume. |

## Gaps (slice 1)

*(Historical: rows 1–2 were closed by the `83ad384f` re-pin below, then re-opened in
part by the SDK migration — see the 0.2.0 gap list.)*

- Placement uses `default_state` of the clicked block (item id = block id); a
  directional block loses its facing, and blocks whose item id differs from the
  block id are not supported. Upstream computes `getStateForPlacement` with a
  fake hit context.
- Only `minecraft:air` counts as placeable (upstream also replaces replaceable
  blocks when the `replace` option is on); no living-entity or block-entity
  blacklist check yet.
- No options screen/undo/keys, no angel/destruction/replacement modes, no
  upgrade components or battery (pin gaps: no energy hook, no component write
  during use; `smithing_upgrade` recipes are inert until native code reads them).
- No `native_play_sound` call yet (one place sound per job upstream), no place
  event, and the stick's durability is not carried by the content items (they
  are plain sticks until `max_damage`/stack-size data is added).

## In-game check

`tools/e2e.sh`: builds a 5×3 stone wall, right-clicks its south face with a
wooden stick (limit 3), and asserts exactly 3 stone appear in the plane and
exactly 3 stone items are consumed. Without the port a stick has no use, so the
check cannot pass vacuously.

**Rewritten for the SDK package (0.2.0):** it now takes `GAME=<launcher-built
horizon-client>` (no `--dev-mod`, no embedded-content leg), an optional
`CONTROL_GAME=` for the vacuity control, and keeps the fail-capable `SELFTEST=1`
and `COUNTERFACTUAL=1` legs. Phase 2 asserts the placed stairs carry the clicked
`facing=north`, and phase 3 asserts a **replaceable** cell (a snow layer) is
filled. Phase 3 of 0.1.0 (a cow blocking a cell) is gone: the SDK exposes no
living-entity query, so the port cannot check obstruction — recorded as a gap.
**Not run in this migration:** `horizon-launcher` is not installed here, so no
launcher-built game exists; the two 0.1.0 transcripts in
`ports/_shared/evidence/constructionstick/` are historical evidence of the same
planner, not of the 0.2.0 package.

## Re-pin `83ad384f` (2026-11-06): the vanilla placement path

**Class + tier** (from `ports/ledger.csv`, per `horizon/docs/modpacks.md`):
class **content** (the five sticks and their models/lang are carried), tier
**logic** (the mod's behaviour is the right-click plane fill). Approach: **agent
port** of the behaviour in Rust + **carry-over** content; no machines/deep
features on this slice.

The slice-1 gaps "placement uses `default_state`" and "only `minecraft:air` is
placeable" are closed. `mixins/constructionstick/support.rs` no longer writes
`registry.default_state` with `native_set_block`; every planned cell is placed
through `Game::with_level` + `Behaviours::placement_state` (dry run, used by
`can_place`) and `Behaviours::place_block` with a real `PlaceContext`
(`clicked_pos` = the planned cell, the clicked face, its centre as the hit, the
player's yaw/pitch/sneak, the clicked block's own item). That gives the block's
vanilla `getStateForPlacement` (a stair keeps a facing), survival and
living-entity obstruction, `setPlacedBy` and the place sound. `runtime.rs` gained
`accepts(cell, resolved)`: a placement that vanilla resolves beside the cell
(an occupied cell) is refused instead of placed out of the plane, and a
`Some(None)` refusal is refused too (`a_placement_must_land_in_the_planned_cell`).

Still a gap (unchanged): the mod's own option set (`replace`, `match`, `lock`,
`direction`), `mayInteract`/spawn protection, the upgrade components/battery,
the options screen/undo, container-sourced supply and the vanilla
`placed_block` trigger. Row 1 of `REPIN-AUDIT.md` is done; rows 3-11 and 13-14
remain later slices.

### Verification (pin `83ad384f48d0742746d518737511882680aab5b7`, mixins `e8cd0aa`)

| Check | Evidence | Result |
|---|---|---|
| embedded test | `target/horizon-test.log` (23:54), `cargo_exit: 0` | `test result: ok. 8 passed; 0 failed` (incl. `a_placement_must_land_in_the_planned_cell`), 0 warnings |
| check | `target/horizon-check.log` (23:48), `cargo_exit: 0` | `Finished dev profile`, 0 warnings |
| fast build | `target/horizon-build.log` (00:15), `cargo_exit: 0` + `ctl_exit: 0` | `Finished fast profile` (24m35s + 16.6s ctl), 0 warnings |

`tools/e2e.sh` now has three phases (survival throughout). Re-run through the
shared build-slot lease at 00:16, `/home/adlf/.cache/opencode/g3/queue.log`:

```text
plane stone after: 3            # phase 1: 3 stone placed, 3 consumed
plane stairs after: 3           # phase 2: stairs at 0/1/-1 101 1 carry
minecraft:stone_brick_stairs[facing=north,half=bottom,shape=straight,waterlogged=false]
cow cell after: minecraft:air   # phase 3: the cow's cell is refused
plane stone after: 3            # the rest of the plane still fills
PASS: the wooden stick filled 3 blocks in the clicked face's plane and consumed 3 stone; stairs kept vanilla facing=north; a cow in a plane cell blocked only that cell
```

exit **0**. The stairs' default state is `facing=east`, so `facing=north` can
only come from the placement path, not from a default-state write.

Counterfactual (the port's item withheld, every step and assertion unchanged,
same log): `stone after: 64`, `plane stone after: 0`,
`PASS (counterfactual, plain minecraft:stick): 0 blocks placed, 0 stone consumed`,
exit **0**.

Delivered bundle: `target/constructionstick-0.1.0.hnmod`, **7,044 bytes**,
SHA-256 `aee0b5843fe9e45688304d2e6a835278afce7ba8ee811af5adacb84f6c030183`.

Harness note: `ctl tp` moved the client only and the server corrected the
player ("moved wrongly") before the click, so the e2e now uses the server-side
`tp @s` command and asserts the resulting position.

Skills: superpowers/using-superpowers,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Upstream source

Analysed 2026-11-02. Repository `https://github.com/Mrbysco/ConstructionSticks`
(MIT), version **1.5.0**, Minecraft 1.21.1 NeoForge 21.1.233, commit
**`a31daed568a41c1e18943697a8a90f005fecf202`** (`main`, the 1.21.1 line; the
repo has no release tags). Areas read: `registry/ModItems.java`,
`items/stick/*`, `items/template/*`, `stick/StickJob.java`,
`stick/StickItemUseContext.java`, `stick/action/*`, `stick/supplier/*`,
`basics/pool/*`, `basics/StickUtil.java`, `basics/ReplacementRegistry.java`,
`basics/option/*`, `stick/undo/*`, `config/ConstructionConfig.java`,
`recipe/SmithingApplyUpgradeRecipe.java`, `data/server/RecipeGenerator.java`,
the generated recipes/tags/lang, and the client keybind/preview code.

The source fixes these facts for the rewrite (site citations in the findings):

- **Items and limits.** Five sticks, `max_stack_size` 1, durability = the
  vanilla tier's uses (59/131/250/1561/2031) and placement limits
  **3/9/27/128/1024**; angel range 1/2/4/8/16; destruction limit 1/3/9/25/81;
  battery 5k/10k/25k/100k/200k with 10 per block. The limit is
  `min(remaining durability, action limit)`; one durability per block; no
  cooldown (`ConstructionConfig.java:119-123`, `StickJob.java:70-77,88-111`).
- **Placement is a plane, not a line.** The default action anchors at
  `clicked + clickedFace`, then breadth-first fills the plane of the clicked
  face across the two unlocked axes (both directions plus diagonals) until the
  tier limit, accepting a cell only when the block behind it matches the clicked
  block under the `match` option (`ActionConstruction.java:40-121`). The `lock`
  option (`nolock` default) restricts which axes are grown
  (`StickOptions.java:118,135-138`).
- **Validation** (`PlaceSnapshot.java:34-126`, `StickUtil.java:221-287`): target
  must be air or replaceable (option `replace`, default on), within `MaxRange`
  100 measured in x/z only, `mayInteract`; placement state must survive, be
  allowed by the block-entity blacklist (`chiselsandbits`, `mekanism`,
  `waystones` by default), and not collide with a living entity. In the default
  `direction = target` mode, orientation properties (`facing`, `axis`, `half`,
  `rotation_16`, `stairs_shape`, non-double `slab_type`) are copied from the
  supporting block; `direction = player` keeps vanilla's orientation.
- **Supply and payment** (`SupplierInventory.java:45-149`): offhand `BlockItem`
  overrides the clicked block; otherwise the clicked block's item plus the
  similar-block set (default `match = similar`; dirt-family group from config).
  Counts are reserved while snapshots are built, then one item is taken per
  placed block from main inventory first (containers supported) and the offhand
  last; if the take fails the block is removed again. Creative pays nothing.
- **Other actions:** Angel places on the far side of the clicked row (stepping
  opposite the face up to its range) and, from air, at
  `round(playerCentre + look*2)` (blocked while falling > 10 unless
  `AngelFalling`); Destruction removes matching blocks on the clicked plane with
  a permeability line-of-sight check and voids them; Replacement swaps matching
  blocks for the offhand block's default state and drops the old block.
- **Undo** keeps the last 3 operations per player; holding the (default-unbound)
  undo key and right-clicking within 3 blocks of the last operation restores it
  and returns the items.
- **Upgrades** are applied in a smithing table by a custom recipe type that
  refuses a duplicate component, an incompatible pair (battery ↔ unbreakable)
  or a stick with `upgradeable = false`.

How the rewrite differs (details and the full pin mapping in
[`../_shared/upstream/constructionstick/FINDINGS.md`](../_shared/upstream/constructionstick/FINDINGS.md)):

- **Plane fill is kept**, but the first slice ships `match = exact`,
  `direction = player`, `lock = nolock`, `replace = on` and no upgrades; the
  option components, `direction = target` property copying and the undo history
  are later slices.
- **Block placement** uses the in-crate path `Game::with_level` +
  `Behaviours::place_block` at the pin (`game.rs:3888`,
  `horizon-world/src/level/place.rs:1746`), so vanilla state selection,
  survival, `setPlacedBy` and the place sound are preserved; durability uses
  `Game::damage_held` (`game.rs:12477`), which already applies Unbreaking.
- **API gaps recorded**: no energy (battery upgrade can't interoperate), no
  component write during use hooks (options need a one-tick write-back or a
  command), no user-bindable keys/options screen, **smithing recipes ignore
  result components at the pin** (upgrades must move to crafting-table recipes
  or wait), no container-sourced items, no item-entity drop for the replacement
  action, and no `mayInteract`/spawn-protection query. All listed in the
  findings' gap table.

## Verification results (in-game)

Proof of slice 1 at the pin `c9d474dd9116e84c243f8992d75c11314df56f06`
(2026-11-02).

### Build

```
$ HORIZON_MIXINS=<campaign>/horizon-mixins \
    python3 tools/horizon.py build --horizon <campaign>/horizon
{ "cargo_exit": 0, "stage_changed": false,
  "binary": ".../target/horizon-cargo/fast/horizon-client" }        exit 0

$ cd target/mixins/modded
$ cargo build --profile fast --locked --offline -p horizon-client \
    --bin horizon-ctl --no-default-features --target-dir .../target/horizon-cargo
    Finished `fast` profile in 2.29s                              exit 0
```

- `horizon-ctl`: `target/horizon-cargo/fast/horizon-ctl`
- `horizon-client`: `target/horizon-cargo/fast/horizon-client`

### Content loading

The native bundle contains only the hooks; the stick items come from the port's
`content/` mod, loaded the same way as simplemagnets:

```
horizon-ctl launch --game .../horizon-client --world ConstructionStickE2E \
  --seed 20261102 --headless --wait --game-dir $SCRATCH/game \
  --resources /home/adlf/Desktop/horizon/.local/horizon-client/26.3 \
  --dev-mod <campaign>/ports/constructionstick/content
```

Without `--dev-mod` the `give constructionstick:wooden_stick` fails. The run is
**survival**, not `--creative`: the port's `support.rs` deliberately places
without payment in creative, so the 3-stone consumption assertion needs
`game.creative == false`.

### Real run — exit 0

`timeout 500 tools/e2e.sh` → exit 0. The rig is a 13×13 stone platform at
`y=99` in open air (the earlier ground-level rig sat in badlands terrain and
falling red sand entered the cleared box), a 5×3 stone wall at `z=0`,
`y=100..102`; the player stands at `(0,100,2)` and aims at the wall's south
face.

```
held: constructionstick:wooden_stick
stone before: 64
plane stone before: 0
== right click the wall with constructionstick:wooden_stick (limit 3) ==
stone after: 61
plane stone after: 3
{"counts":{"minecraft:stone":3},"blocks":[
  [-1,101,1,"minecraft:stone"],[0,101,1,"minecraft:stone"],[1,101,1,"minecraft:stone"]]}
PASS: the wooden stick filled exactly 3 blocks in the clicked face's plane and consumed 3 stone
```

Exactly 3 blocks in the clicked plane (`z=1`, `y=101`) and exactly 3 stone
consumed, as asserted (the script fails on any other delta).

### Counterfactual — exit 0, and it proves the check can fail

`COUNTERFACTUAL=1 tools/e2e.sh` runs the identical scenario with a plain
`minecraft:stick` (the port's item withheld, every other step and assertion
unchanged):

```
held: minecraft:stick
stone before: 64
plane stone before: 0
== right click the wall with minecraft:stick (limit 3) ==
stone after: 64
plane stone after: 0
PASS (counterfactual, plain minecraft:stick): 0 blocks placed, 0 stone consumed
```

Same click, same wall, zero blocks and zero items: the placement and the payment
both come from this hook.

### Script fixes made while verifying

- `look-at-block` was sent through `ctl command` (a chat command) instead of
  being called as a `horizon-ctl` subcommand, so the click was never aimed;
- `plane_stone` filtered `.[2]` (z) where it meant y, and counted the whole
  box, so it could never see the plane;
- `--creative` made `support.rs` place for free, contradicting the 3-stone
  assertion;
- `ctl give ... || fail=1` was never checked; content/item/held-item are now
  hard assertions.

## Re-pin `1d63e0a5` and the one-file bundle (2026-11-10)

**Class + tier** unchanged (class **content**, tier **logic**). The port stays an
agent port of the behaviour plus carried content, and it is now a **single
`.hnmod` deliverable**: the built content mod travels inside the bundle at
`mixins/constructionstick/data/constructionstick-content.hmod` and one
`embedded-content` injection at `horizon_client::game::mods::content_root` writes
it to `.horizon-cache/` and appends it to the dev list (the AE2 pattern,
`docs/native-mods.md` "Data files"; proved on `charginggadgets`, PR #526). The
registry keeps one file per project, so a split port could not be listed.

- Pin: Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` (`game.rs` SHA-256
  `64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f`), mixins
  `5a64af938ec86b00d53d16a408aabc0038fb3e57`. `compatibility.json` and
  `horizon-mixin.toml` agree; `tools/horizon.py` defaults to the pin checkouts
  and runs the embedded tests with `--lib game::constructionstick::tests`.
- Artifacts: `content/constructionstick.hmod` **86,208 B**, SHA-256
  `3f1cd6f34fceed3a7370358f4d68d405dadda861547af2fd013605e495596d3f`, 157
  entries, `no code: content only`; `constructionstick.hnmod` **62,907 B**,
  SHA-256 `a9dd66b2381cadb2219232aff98c8407c7fbe6ac85c2cb5c542b9dd0efd69f94`,
  7 entries (manifest + 5 sources + the embedded content mod);
  `cargo horizon-mod check … --kind native-mod` exits 0.
- Ladder: `cargo fmt -- --check` clean, `cargo clippy --all-targets -- -D
  warnings` clean, `cargo test --locked --offline` 8 passed, `horizon.py
  stage/check/test/build` `cargo_exit: 0` with **0 warnings**, 8 embedded tests
  under `game::constructionstick::tests`, fast build.
- In-game, `EMBEDDED=1` (the default) launches **with no `--dev-mod`**:
  control leg first, with `.horizon-cache` made unusable — no construction stick
  can be given — then the three phases (3 blocks/3 stone, stairs keep
  `facing=north`, a cow blocks only its cell). `PASS` + `EXIT:0`
  (`ports/_shared/evidence/constructionstick/e2e-embedded.log`).
  `SELFTEST=1` asserts 4 blocks and fails: `FAIL: the click placed 3 blocks,
  expected 4`, `EXIT:1` (`e2e-selftest.log`).
- Harness note: the game's working directory is the `--game-dir` game folder, so
  the embedded content lands in `<game>/.horizon-cache/`; the control makes
  **that** path unusable, not the shell's cwd.

## Review repairs (PR #626)

1. Staging is the orchestrator's (horizon-mods#686).
2. *"Fix the original-project credit link … use
   `https://github.com/Mrbysco/ConstructionSticks`"* — done, with Mrbysco
   credited.
3. *"The glue now calls the vanilla placement path, so the registry description
   saying it uses only `default_state` and lacks entity-obstruction checks is
   stale and should be updated"* — the description now says placement goes
   through the game's vanilla path (a directional block keeps its facing) and
   lists entity-obstruction/replaceable-block handling as the actual gap.
4. The notices travel inside the artifact (`mixins/constructionstick/LICENSE`
   and `NOTICE.md`, and at the embedded content root).

### Verification after the repairs

- Artifact: `constructionstick.hnmod`, 65,668 B, SHA-256 `979a113d17626f726018442fbac68e28e6db38362e7169483fee34c91dc03a94` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/constructionstick.hmod`).
- `python3 ports/tools/upstream_ready.py constructionstick` and `check-skills.py --notes ports/constructionstick`: exit 0.

## Migration to the SDK format (0.2.0, 2026-11-18)

*Migrated from the retired `.hnmod` container to the flat SDK package, following
`ports/imfast/notes.md` → `## Migration to the SDK format` (the verified 12-step recipe
and 15 traps), with `ports/buildinggadgets2/notes.md` as the closest shape (an item Part
that places blocks and pays for them) and `ports/naturesaura/notes.md` for the Parts and
manifest traps. Only the **glue, the manifest and the packaging** were rewritten:
`src/runtime.rs` and its 8 tests carried over byte-for-byte, and every carried content
file is unchanged.*

**Why.** Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`
on 2026-11-18: the registry closed every `.hnmod` submission with one reason ("republish
this as a native `.hmod` with the SDK from the next Nightly"), and an `.hnmod` no longer
loads. The new unit is **one flat `.hmod`** carrying content and code together — which is
what this port already faked with the AE2 embedded-content pattern
(`mixins/constructionstick/data/constructionstick-content.hmod` + a `content_root`
injection). That embedding is gone: `assets/` and `data/` sit at the package root and the
launcher loads both halves from the one file.

**Pins.** SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18),
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (moved from `5a64af93…`),
`package_format` 6, `content_api` "0.2", declared toolchain rust 1.99.1. The SDK was
copied in from `ports/framedblocks/.horizon/sdk` (same pin, 21 MB) so every build here ran
offline; `.horizon/` is git-ignored and never packed.

**Tool.** `cargo horizon-mod` runs **directly in the port** with `CARGO_INCREMENTAL=0`
and no `--target-dir`. `ports/tools/build-slot.sh run --job constructionstick-sdk --port
ports/constructionstick --step <test|lint|fmt|build|check>` is still used as the **slot
guard** (with `BUILD_LOCAL=1`, since there is no staging left to forward), but the ladder
inside it is the SDK one; the old `tools/horizon.py stage|check|test|build` is retired with
the format.

### Done in this order

1. The retired layout was **moved** to
   `ports/_shared/evidence/constructionstick/retired-hnmod-0.1.0/` **before the first
   build** (the tool refuses a root holding retired package paths). Moved: `mixins/`
   (`horizon-mixin.toml`, `runtime.rs`, `support.rs`, `use_on.rs`, `embedded_content.rs`,
   `data/`), `constructionstick.hnmod` (65,668 B, `979a113d…`), `compatibility.json`,
   `rust-toolchain.toml`, `content/` (its `assets/`, `data/`, `patches/`, `README.md`,
   `horizon-mod.toml` and the built `constructionstick.hmod` 87,549 B `1df75d26…`),
   `tools/{horizon,package,test_stage,test_template,add-notices}.py`, `tools/package.sh`,
   `tools/audit/`, `.github/`, `Cargo.lock` (deleted — never packed), and the port's
   `AGENTS.md` (it described the retired `.hnmod` workflow, which this layout
   contradicts; the campaign root's `AGENTS.md` governs, as in `buildinggadgets2`).
2. `mixins/constructionstick/runtime.rs` → `src/runtime.rs` with `cp`, proved with `cmp`
   and `sha256sum`: **`3acc74b1…e5230`**, byte-identical, also inside the package
   (`unzip -p constructionstick.hmod src/runtime.rs | sha256sum` → the same).
3. `cargo test --offline` on the new crate **before any glue existed**: **8 passed, 0
   failed** (`evidence/constructionstick/sdk-test-runtime-first.log`).
4. The glue's two pure helpers were written **test-first** and watched fail
   (`error[E0425]: cannot find function 'worn' in this scope`, recorded as an excerpt in
   `evidence/constructionstick/sdk-test-red-excerpt.log`), then implemented:
   **13 passed, 0 failed**.
5. Manifest, README and licence notice written, then packaged, checked, fitted, and built
   twice (the second from a deleted `target/`): identical bytes.

### File map (0.1.0 `.hnmod` → 0.2.0 SDK package)

| Old | New |
|---|---|
| `horizon-mixin.toml` (`[mod]`, `[compatibility]`, 2 `[[module]]`, 2 `[[injection]]`) | `horizon-mod.toml` only: `id`, `name`, `version = "0.2.0"`, `description`, `authors`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native]` `game`/`horizon`/`mixins` |
| `Cargo.toml` (`[lib] path = "mixins/constructionstick/runtime.rs"`, `[workspace] members = ["tools/audit"]`) | `Cargo.toml` (`[lib] path = "src/lib.rs"`, one dependency: `horizon-mod-api` from `.horizon/sdk/`) |
| `mixins/constructionstick/runtime.rs` (module *and* crate lib) | `src/runtime.rs` — **byte-identical**, declared `pub mod runtime;` at the crate root |
| `mixins/constructionstick/support.rs` (`--module constructionstick_state`) | `src/support.rs` — the glue: the `LevelWorld`, the supply count/take, the placement loop, the wear, the Part and the registration |
| `mixins/constructionstick/use_on.rs` (one helper) | `impl ItemPart for Stick` → `fn use_on` in `src/support.rs` |
| `mixins/constructionstick/embedded_content.rs` + `data/constructionstick-content.hmod` (the AE2 write-out) | **gone**: code and content are one package |
| `content/assets/` (51 files), `content/data/` (105 files) | `assets/`, `data/` at the package root — every file unchanged (size + `sha256sum` diffed against the retired copy, `diff -r` empty) |
| `content/patches/extra/**` (the carry overlay) | **not copied**: every overlay file is already byte-identical in `content/data/` (verified with `diff -r`), so it is carry tooling, not package content |
| `compatibility.json`, `rust-toolchain.toml` | **gone**: `[native]` is the pin, and the launcher + pinned SDK decide the toolchain |
| `tools/horizon.py`, `tools/package.py`, `tools/package.sh`, `tools/test_stage.py`, `tools/test_template.py`, `tools/add-notices.py`, `tools/audit/`, `.github/` | **gone** (in evidence); `cargo horizon-mod build` packs, `cargo test`/`clippy` compile |
| `constructionstick.hnmod` (65,668 B) | `constructionstick.hmod` (**102,127 B**, 165 entries) |
| `README.md` (the template's own README) | rewritten for this port (it travels **inside** the artifact) |
| `LICENSE-NOTICE.md` (described the `.hnmod` packaging) | rewritten: the notices ride at the package root |

**Kept:** `tools/e2e.sh` (rewritten for a launcher-built game), `notes.md`,
`REPIN-AUDIT.md`, `VALIDATION.md`, `LICENSE`, `LICENSE-NOTICE.md`.

### Helper → hook table

| 0.1.0 (`[[injection]]` / helper) | 0.2.0 SDK | Mapping notes |
|---|---|---|
| `[[module]] runtime` → `crate::game::constructionstick` | `pub mod runtime;` in `src/lib.rs` | an ordinary module now; the file is untouched |
| `[[module]] state` → `crate::game::constructionstick_state` | `src/support.rs` (`pub mod support;`) | the planner's `World` impl, the payment and the wear |
| `[[injection]] use-on` (`use_item_on` v1, HEAD, `use_on.rs`; `claimed = true`) | `#[mod_api::horizon_part] impl ItemPart for Stick { fn use_on(&self, host: HostSide<'_>, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> }`, attached in `Hooks::register` with `registrar.item_part(key, Stick)` for the five ids | **the item-scoped `ItemPart::use_on`, not the block-scoped `Hooks::use_item_on`** (buildinggadgets2 trap 23): the game asks it only for those items, so the "is the stack a stick" check is only the tier lookup; `claimed = true` becomes `Some(InteractionResult::SuccessServer)`; the client half answers `None`, so a client prediction never writes the world |
| `[[injection]] embedded-content` (`content_root` HEAD) | **none** | one package, nothing to write out |
| `game.native_acting_player()` | `host.player(event.player)` | `UseOnEvent.player` is the clicking player's UUID on each side |
| `game.creative` | `PlayerRef::is_creative(host)` | creative places for free, as before |
| `game.pack.assets.registry.default_state(name)` / `.state(id)` | `LevelRef::block_state(host, pos)` (the **clicked state**) and `LevelRef::block(host, pos)` → `BlockKey::id(host)` | the state to copy is now read from the world, not from a name |
| `game.pack.assets…registry.state(...).item` (block → item) | `host.block_item(block_key)` | the item the player pays with; `minecraft:air` (no item) refuses the click |
| `game.native_count_items` / `native_take_items` | `PlayerRef::inventory_slots(host)` filtered to `slot < 36`, and `PlayerRef::set_inventory_slot` to take | explicit; also sees component-bearing stacks |
| `game.with_level(behaviours.placement_state)` + `Behaviours::place_block` (the vanilla placement path) | **`PlayerRef::place_block(host, level, cell, state, false)`** | the closest supported shape: `setBlock` with a placement's flags plus `Block.setPlacedBy` in the player's turn. Not `BlockItem.place` — see the gaps |
| `game.native_set_block(&dimension, cell, air, UPDATE_ALL)` (the rollback) | `LevelRef::set_block(host, cell, air, BlockUpdateFlags::NEIGHBORS \| CLIENTS)` | `UPDATE_ALL` is those two bits |
| `game.damage_held(1)` | `host.max_damage(item)` + `ItemStack::damage`/`set_damage` + `PlayerRef::set_inventory_slot` | there is no damage helper; the loop carries the damage forward (`stack.damage()` never changes) |
| `game.native_feedback` / nothing (the slice had no command) | nothing | the port declares no command; `perform_command`'s local `run_mod_command` rule does not apply here |
| `game.player.dimension` | `PlayerRef::level(host)` | a `LevelRef` handle, and `place_block`/`set_block` go through it |

**Hooks mapped, gaps recorded:** one behavioural hook (`use_on`), one declaration
(`Registrar::item_part` ×5), and **no hook without a home** — the embedded-content
injection's job is done by the format itself. `uses.json` (126 entries) confirms it:
`hook:use_on` (+ its five fields), `method:ItemPart::use_on`,
`method:Hooks::register`, `method:ItemsRegistrar::item_key`,
`method:HostBlockStates::block_states`, the `BlockStatesHost`/`ItemsHost`/`PlayersHost`/
`EntitiesHost` calls (`block_item`, `block_state_id`, `max_damage`, `level`, `player`),
the handles (`BlockKey::id`, `BlockState::block`, `BlockStates::replaceable`,
`LevelRef::{block, block_state, set_block}`, `PlayerRef::{level, inventory_slots,
selected_slot, set_inventory_slot, place_block, is_creative, is_spectator}`,
`ItemKey::id`, `ItemStack::{item, count, damage, set_damage, set_count}`,
`BlockUpdateFlags::{NEIGHBORS, CLIENTS}`), both macros and the `registries` capability.
The report also lists the declaration hook `blocks` with its default no-op body — nothing
is declared in code (the blocks/items are data files), and `naturesaura`'s package is the
same.

### Placement: what replaced the vanilla path

The 0.1.0 port placed every cell through `Behaviours::place_block` with a real
`PlaceContext`, which gave it `getStateForPlacement` (a stair kept a facing), survival and
entity obstruction, `setPlacedBy` and the place sound. **There is no host call for that at
this pin.** The nearest are:

- `PlayerRef::place_block(player, level, pos, state, update_neighbour_shapes)` — what the
  port uses: `Level.setBlock` with a placement's flags, then `Block.setPlacedBy` run in
  the player's turn (a bed places its head, a door its upper half). The docs are explicit
  that nothing else of `BlockItem.place` runs.
- `LevelRef::set_block(host, pos, state, flags)` — what the rollback (and
  `buildinggadgets2`) uses.
- `host.edit_blocks(dimension, &BlockEdit)` — a bulk `fill … strict`, the wrong shape
  here: the fill must stop when the supply runs out, and it notifies no neighbour.

The **state** copied into each cell is therefore the **clicked state**
(`level.block_state`), not the mod's own `getStateForPlacement`: a stair keeps its facing
(phase 2 of the e2e), a slab its half, and the copied block is exactly the one clicked.
Upstream's `direction = target` property copy and `direction = player` remapping stay
later slices, as they were in 0.1.0.

`runtime::accepts` is kept and now structurally satisfied: the SDK writes the given state
in the planned cell, so a placement can no longer resolve beside it. It stays a `pub fn`
in the `pub mod runtime` (no dead-code warning), and the runtime file stays byte-identical.

### Gaps (0.2.0)

- **No entity obstruction.** `level.entities_in(level, min, max)` returns `EntityRef`s and
  nothing turns one into a `LivingRef` (`EntityRef` has no `living()`), so
  `runtime::World::can_place` is "loaded and replaceable" only: a cow standing in a plane
  cell no longer blocks that cell (0.1.0 got this from vanilla's placement path, and its
  e2e phase 3 proved it). **Issue shape:** *"A query for the living entities overlapping a
  box (`EntityRef::as_living()` or `level_living_entities_in`), so a builder can skip an
  obstructed cell."*
- **No `canSurvive` query** (the same gap `buildinggadgets2` recorded): a state that cannot
  stand where it is written is written and then dropped by the game's shape updates, where
  0.1.0 skipped the cell. **Issue shape:** *"A host query for `BlockState.canSurvive(state,
  pos)`."*
- **No `getStateForPlacement`**: the clicked state is copied instead (above).
- **No place sound, game event, statistic or `placed_block` advancement trigger**, and
  `setPlacedBy` receives no placer — `player_place_block`'s documented limits.
- **No `mayInteract`/spawn-protection check** (`player_may_use_item_at` exists at this pin;
  it was a 0.1.0 gap too and stays a later slice).
- **Durability is written but never trips**: the content's items are
  `{"template": "minecraft:stick"}`, so `max_damage` is 0 and nothing wears (`worn` is
  unit-tested for the day the data declares a durability). No Unbreaking either — a damage
  write is not `damage_held`.
- **The mod's option set** (`match`, `lock`, `replace`, `direction`), the options
  screen/undo, the angel/destruction/replacement modes, the upgrades/battery and
  container-sourced supply remain later slices (unchanged from 0.1.0).

Closed by this migration: **replaceable cells are filled** (`BlockStates::replaceable`, so
upstream's `replace = on` default holds), where 0.1.0 accepted `minecraft:air` only.

### Deviations from 0.1.0 (deliberate)

1. **The plane's state is the clicked state**, not vanilla's `getStateForPlacement`
   result (see above). For the e2e's north-facing stair wall both give `facing=north`;
   for a stair clicked on its own south face, 0.1.0 gave the player's facing and this
   gives the clicked stair's.
2. **The supply is counted and taken from the main inventory only** (`slot < 36`), as
   upstream pays from the main inventory first; the off hand is not payment.
3. **The off hand is handled too**: a stick used from the off hand wears slot 40
   (`hand_slot`); 0.1.0's `damage_held` was the main hand.

### Traps this migration hit (in addition to the reference's 22)

1. **`content/` was not only `assets/` + `data/`.** It also held the *built*
   `constructionstick.hmod` (87,549 B) and a `patches/extra/` overlay. Moving `content/`
   wholesale into the root would have shipped a stray binary and duplicated ids; the
   overlay is already applied (every patch file is byte-identical in `data/`, `diff -r`
   empty), so only `assets/` and `data/` were copied.
2. **`support.rs` needed no rewrite of its header**, contrary to the old mixin rule: the
   "no `//!` in support" rule came from the mixin planner (`E_MODULE`). In the SDK the
   file is an ordinary module, so it now *should* carry a `//!` header like the rest of the
   crate.
3. **A hook that hands `&ItemStack` cannot mutate it.** The stick's wear is a
   read-modify-write through `player_set_inventory_slot`, and because `stack.damage()`
   never changes, the placement loop must **carry the damage forward itself** — otherwise
   every placed block writes damage 1.
4. **`max_damage` of a template item is 0.** `{"template": "minecraft:stick"}` has no
   durability, so the wear path is inert in game; it is unit-tested rather than dropped,
   and the gap says why.
5. **There is no `damage_held`, no `ItemStack::hurt`, and no placer-aware place.** The
   three host calls that exist (`place_block`, `set_block`, `edit_blocks`) each give up a
   different part of `BlockItem.place`; the notes must state which parts, not just "uses
   the vanilla path".
6. **`player.place_block` refuses `Unchanged`.** A cell the game considers unchanged comes
   back as an `Err`, so "skip and keep filling" (0.1.0's answer to a vanilla refusal) must
   treat *every* `Err` as a skip — `break` would have stopped the plane at the first
   occupied cell.
7. **Clippy's `too_many_arguments` (8/7) fires on the placement function** and `-D
   warnings` makes it fatal: group the per-use data (`Placement { state, item, hand,
   stack }`) rather than allow the lint.
8. **`cargo horizon-mod check` prints a *caveat*, not an error**: "native source:
   unsandboxed; not executed by the registry check". The compile gate is `cargo test` +
   `clippy -D warnings`; `check` only validates the manifest and the archive.
9. **The `check --json` form reports what the checker resolved, not the manifest** (the
   reference's buildinggadgets2 trap 26): do not "fix" the manifest for it.
10. **The e2e lost a phase with the format.** Its old phase 3 proved entity obstruction,
    which is now impossible; leaving the assertion would have produced a script that can
    only fail. It is replaced by the replaceable-cell phase (a behaviour this migration
    *adds*), with the obstruction gap stated in the header — a rewrite of the check to
    match the port, not a weakened assertion.

### Verification (this migration)

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

| Check | Command | Result |
|---|---|---|
| `runtime.rs` carried | `cp` + `cmp` + `sha256sum` | byte-identical, **`3acc74b137ddd8294fb922af73574813a64a85e38ccb7e66b3efd596644e5230`**, also inside the package (`unzip -p … src/runtime.rs \| sha256sum`, same) |
| Runtime tests before any glue | `CARGO_INCREMENTAL=0 cargo test --offline` | **8 passed**, 0 failed (`evidence/…/sdk-test-runtime-first.log`) |
| Glue red first | same command, glue declared nothing | `error[E0425]: cannot find function 'worn' in this scope` (excerpt: `sdk-test-red-excerpt.log`) |
| Tests | `cargo test --offline` | **13 passed**, 0 failed (`sdk-test.log`) |
| Lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| Formatting | `cargo fmt --check` | clean, `fmt_exit:0` (`sdk-fmt.log`) |
| Package | `rm -f Cargo.lock; cargo horizon-mod build` | `target/horizon-mod/constructionstick.hmod`, exit 0 (`sdk-build.log`) |
| Registry check | `cargo horizon-mod check constructionstick.hmod` | exit **0** (`sdk-check.log`, quoted below) |
| Fits the build | `cargo horizon-mod fits constructionstick.hmod --build nightly` | `0b14f8a  made for`, exit 0 (`sdk-fits.log`) |
| Determinism | `rm -rf target && rm -f Cargo.lock && cargo horizon-mod build` again | same **102,127 B**, same sha256 both times (`sdk-build-second.log`, `sdk-sha256.log`) |
| What it uses | `unzip -p constructionstick.hmod uses.json` | 126 entries (`sdk-uses-summary.log`) |
| Package contents | `unzip -Z1` | **165 entries**: `horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `src/{lib,runtime,support}.rs`, `uses.json`, 51 `assets/…`, 105 `data/…`; **no `Cargo.lock`**, no `notes.md` |
| Symbol line | `python3 ports/tools/check-skills.py --notes ports/constructionstick --strict` | exit 0 |

`check` output, verbatim:

```text
constructionstick 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  102127 bytes, SHA-256 0660f04c2ed5b36966646845e7d81211218a102ada9cebd7b24f58de083f5136, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.
```

`fits` output, verbatim:

```text
Construction Sticks:
0b14f8a  made for
```

### Not re-run in this migration

- **The in-game e2e.** A native mod is compiled into `horizon-client` by the launcher, and
  `horizon-launcher` is not installed in this environment (the reference's trap 15, hit
  again). `tools/e2e.sh` is rewritten for a launcher-built game (`GAME=`, optional
  `CONTROL_GAME=`, `SELFTEST=1`, `COUNTERFACTUAL=1`) and must be run — default,
  counterfactual and selftest — before this port claims `ported`. The 0.1.0 transcripts
  (`evidence/constructionstick/e2e-embedded.log`, `e2e-selftest.log`; both still pass
  `check-e2e-logs.py`) are **historical**: they prove the same `runtime.rs` fills the plane,
  pays 3 stone and refuses an obstructed cell, on the retired format. They are not evidence
  for the 0.2.0 package, whose placement path changed (`player.place_block`, copied clicked
  state, no obstruction check).
- **The staged ladder** (`tools/horizon.py stage|check|test|build`) — those tools are
  retired with the format.

## Changelog (0.2.0) — supersedes #868

> **Construction Sticks 0.2.0** — republished as a native `.hmod` with the SDK (mod API
> 0.2). This supersedes the 0.1.0 `.hnmod` submission (**PR #868**), which the registry
> closed on 2026-11-18 because the `.hnmod` container was retired and no longer loads.
>
> **What changed for `api = "0.2"`:** one flat package — the manifest is
> `horizon-mod.toml` (`api = "0.2"`, `capabilities = ["registries"]`, `[native]` with the
> exact `game`/`horizon`/`mixins` commits) instead of `horizon-mixin.toml` +
> `horizon-native.json` + per-hook helper files; the single behaviour is one
> `#[mod_api::horizon_part] impl ItemPart` `use_on` attached to the five sticks instead of
> a `use_item_on` HEAD injection; the `content_root` embedded-content injection is gone
> because the sticks' `assets/` and `data/` now travel in the same file as the code; and
> the pure planner (`src/runtime.rs`, its 8 tests) is byte-identical. Behaviour is
> unchanged: a right click with a stick fills the clicked face's plane with copies of the
> clicked block, bounded by 3/9/27/128/1024 blocks and by the supply, paying one item and
> one durability per block, and setting the block with a placement's flags plus
> `setPlacedBy`. Two measured differences from 0.1.0: a **replaceable** cell is now filled
> (upstream's `replace = on`), and the SDK has no living-entity query, so a cow no longer
> blocks a cell (recorded as a gap with an issue shape). Known gaps: entity obstruction,
> `canSurvive`, `getStateForPlacement`, place sound, `mayInteract`, the option set, the
> angel/destruction/replacement modes, the upgrades/battery and the options screen/undo.
> The in-game check has **not** been re-run (no `horizon-launcher` here); see
> "Not re-run in this migration". Licence: MIT, with the original notice inside the
> package.
