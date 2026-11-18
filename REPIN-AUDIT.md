# Construction Sticks — re-pin audit at Horizon `83ad384f`

Read-only audit. No port code, build, e2e or campaign document was changed.

- Pin: Horizon **`83ad384f`** (427 hooks / 159 helpers) at `ports/_shared/pins/horizon-83ad384f`;
  mixins `e8cd0aa` at `ports/_shared/pins/horizon-mixins-e8cd0aa`.
- Compared against the port's pin `c9d474dd…` (108 hooks, 0 listed helpers).
- Sources: `notes.md` §Gaps (slice 1) and ARCHITECTURE.md §7 ("Deferred: angel/destruction/
  replacement modes, upgrades, options/undo, battery, place sound/events").
- All paths are relative to `ports/_shared/pins/horizon-83ad384f/`.

The port's only hook (`use_item_on` v1) is unchanged. Two column names matter below:

- **new at this pin**: the hook is absent from `c9d474d`'s `native-hooks.json`, which the planner
  validates (`crates/horizon-mixins-core/src/native.rs:88-97`), or the helper function does not exist
  in `c9d474d`'s tree at all.
- **already available**: the function/hook existed at `c9d474d`; the port simply never used it. This
  audit does not credit those to the re-pin.

## Deferred features

| # | Deferred feature | Pin surface (symbol, file:line) | Class | Verdict | Exact next step |
|---|---|---|---|---|---|
| 1 | Placement loses `getStateForPlacement`: default state only, directional blocks lose facing, item id ≠ block id unsupported | World API `Behaviours::place_block` `crates/horizon-world/src/level/place.rs:1744`, `placement_state` `:1777`, `state_for_placement` `:1790`, `PlaceContext` `:21-40`; hook `state_for_placement` (replace) `.../native_hooks.rs:1727`, call site `game.rs:1435` | usable (already available) | **realizable now** | Stop using `game.pack.assets.registry.default_state` in `support.rs:18`; per cell build `PlaceContext { clicked_pos: cell, face, hit, yaw, pitch, sneaking, item, can_use_game_master_blocks }` and call `game.with_level(…)` with `Behaviours::placement_state` / `place_block` (the closure shape of `game.rs:6944`). Run it once with a stairs target and assert the `facing` property. |
| 2 | Only `minecraft:air` is placeable: no `replace` option, no living-entity / block-entity blacklist | `Behaviours::place_block` resolves `replace_clicked` via `can_be_replaced`, checks survival and entity obstruction and returns `PlaceOutcome::Failed` (`place.rs:1740-1748`); no public `is_replaceable` helper beyond it | usable | **realizable now** | Replace `GameWorld::can_place` (`support.rs:49-51`) with a dry `placement_state` per cell: `Some` = placeable (air or replaceable), `None` = refused (this also gives the survival/entity check). Keep the block-entity blacklist as the mod's own name list. |
| 3 | Options screen, keybinds, undo | `native_mod_screen_open` `.../native_hooks.rs:8968` (fn existed at `c9d474d`, listed as a helper only here; `native_mod_screen` `:8998`, `native_mod_screen_close` `:8991`); draw `native_screen_render_post` `:3372`, `native_fill` `:8823`, `native_text` `:9101`; clicks `screen_click` (replace) / `screen_mouse_button_pressed_pre` (cancel); binding `native_mod_key` `:9159`; keys `key` (listen) `:2569`; history can live in level/player attachments (`native_attachments` `:8406`) | usable (already available) | **realizable now** | Add a `screen` injection + one `native_mod_screen_open("constructionstick")` on the bound key; the options are the five flags from `StickOptions`, the undo stack the last 3 `(pos, old_state)` lists in an attachment. TDD the undo ordering in `runtime.rs`. |
| 4 | Angel mode | `native_raycast` `.../native_claimed_items.rs:137` (fn exists at `c9d474d`, listed as a helper here); place path as rows 1-2; `native_count_items` `.../native_server.rs:637` | usable (already available) | **realizable now** | Add the action to the existing `use_item_on` helper behind the stick's mode component; step opposite the face with `native_raycast` and `native_block`, cap at 1/2/4/8/16. |
| 5 | Destruction mode | `native_set_block` `.../native_hooks.rs:8464`, `native_raycast` `.../native_claimed_items.rs:137`, `native_level_block_state` `.../native_level_refs.rs:42` | usable (already available) | **realizable now** | Plane walk + line-of-sight raycast per cell, `native_set_block(air)`; tiers 1/3/9/25/81. |
| 6 | Replacement mode | `native_set_block` `:8464`; item drop `native_summon("minecraft:item", …)` `.../native_entities.rs:141`; offhand via `native_carried_stack` `.../native_hooks.rs:8450` (slot `-106`) | usable (already available) | **realizable now** | Swap matching cells to the offhand block's `default_state` (or its `state_for_placement` from row 1) and summon the old block as an item entity. |
| 7 | Smithing **upgrades** (result components ignored) | **`prepare_smithing` (replace `result`) `.../native_hooks.rs:5649` — new hook (API 292)**; `native_modify_default_components` (old), `component_types` (old) | usable (new at this pin) | **realizable now** | Keep the smithing recipe type; in the `prepare_smithing` injection write the upgrade component onto the stick stack (`result: &mut Option<ItemStack>`), refusing duplicates and battery↔unbreakable. e2e: smith a wooden stick + upgrade item, assert the option component. |
| 8 | Battery upgrade / energy (pin had no energy) | **`item_energy_buffer` hook (replace `buffer`) `.../native_hooks.rs:3093` — new (API 307)**; helpers `native_item_energy` / `_insert` / `_extract` `.../native_item_energy.rs:156,168,180` | usable (new at this pin) | **realizable now** | Declare a buffer for the five stick ids (capacity 5k/10k/25k/100k/200k, 10 per block) and drain with `native_item_energy_extract` in the placement helper; the amount lives in `minecraft:custom_data.horizon_energy`. |
| 9 | No component write during use (options live in components) | `native_carried_stack(slot) -> Option<&mut ItemStack>` `.../native_hooks.rs:8450` (reads `self.inventory`, the acting player's state is swapped in for `use_item_on`); `native_inventory_tick` `:7609` | usable (already available) | **realizable now** | Write the option component on the held stick via `native_carried_stack(<selected slot>)` inside the existing `use_item_on` helper (or tick it in `inventory_tick`). |
| 10 | No container-sourced supply | `item_container` hook `.../native_hooks.rs:3109` + `native_item_container_slots/insert/extract` `.../native_item_container.rs:166,181,194` (new, API 370); block containers `native_container_slots` `.../machines/native_items.rs:116`, `native_extract_item` `:142` (already available) | usable (new at this pin) | **realizable now** | Extend the supply step: when `native_take_items` is short, pull the same item out of the player's shulker/backpack items (`item_container_*`) before failing the placement; the block-container helpers cover containers in the world. |
| 11 | No item-entity drop for replacement | `native_summon` `.../native_entities.rs:141` (fn exists at `c9d474d`, listed as a helper here) | usable | **realizable now** | See row 6. |
| 12 | No `mayInteract` / spawn-protection query | No symbol in this pin (grep `mayInteract`, `spawn_protection`, `may_build` over `game/native_*` finds nothing) | absent | **blocked** | Keep the current behaviour; pin request for a `Level.mayInteract`-style query. |
| 13 | No place sound and no place event | Sound: `native_play_sound` `.../native_hooks.rs:8193` (already available). Event: vanilla's `placed_block` trigger is not exposed; `native_trigger_criterion` `.../native_criteria.rs:61` only fires a mod's own criteria | usable (sound) / absent (vanilla trigger) | **partial — realizable now for the sound** | Play one `minecraft:block.<material>.place` per job with `native_play_sound`; do not claim the vanilla advancement trigger. |
| 14 | Stick durability / stack size not on the content items | Item `components` patch in content (`crates/horizon-mods/src/content.rs:330-336`), e.g. `minecraft:max_damage`, and `max_stack_size` | usable (already available) | **realizable now** | Add `"max_stack_size": 1` and `{"minecraft:max_damage": 59}` (131 / 250 / 1561 / 2031 for the other tiers) to the five item files; the existing `damage_held` (`game.rs:12477`) then wears them and Unbreaking applies. e2e: the stick's durability bar drops after a job. |

## Summary

- **Deferred features audited:** 14.
- **Newly realizable because of the re-pin:** 3 — **row 7** (`prepare_smithing`), **row 8**
  (item energy) and **row 10** (item containers). These were impossible to declare at `c9d474d`
  (`prepare_smithing`, `item_energy_buffer`, `item_container` are absent from its
  `native-hooks.json`; the energy/container helpers did not exist in its tree).
- **Realizable now with capabilities already present at `c9d474d`:** 9 — rows 1, 2, 3, 4, 5, 6, 9,
  11, 14. Rows 1-2 are the most important and cheapest: the port's placement helper bypasses the
  vanilla placement rules that `Behaviours::place_block`/`placement_state` already implement.
- **Partial:** 1 — row 13 (sound yes, vanilla `placed_block` trigger no).
- **Still blocked:** 1 — row 12 (`mayInteract`/spawn protection).
- **Highest-value next step:** replace the `default_state` + `native_set_block` loop in
  `mixins/constructionstick/support.rs` with a per-cell `Behaviours::placement_state` /
  `place_block` through `game.with_level(…)`. It fixes three recorded gaps at once (facing/property
  loss, `replace` + entity/block-entity safety, durability of the placed block's state) and makes
  rows 4-6 (angel/destruction/replacement) a thin layer on top. It is testable without the pin's
  new APIs and fail-capable (stairs must keep `facing`, a cell holding a cow must be refused).
  Then take row 7 (`prepare_smithing`) as the first repin-only slice, because upgrades are the
  feature players notice.

## How to re-check (read-only)

```sh
P=ports/_shared/pins/horizon-83ad384f
python3 - <<'EOF'
import json; d=json.load(open('ports/_shared/pins/horizon-83ad384f/native-hooks.json'))
old=json.load(open('/tmp/…/hooks-old.json'))  # or: git -C horizon show c9d474d:native-hooks.json
for n in ('prepare_smithing','item_energy_buffer','item_container','state_for_placement','place_block'):
    print(n, [h['mod_api'] for h in d['hooks'] if h['name']==n])
EOF
grep -n "pub fn place_block\|pub fn placement_state\|pub fn state_for_placement\|pub struct PlaceContext" \
  $P/crates/horizon-world/src/level/place.rs
grep -n "fn native_prepare_smithing\|fn native_item_energy_buffer\|fn native_mod_screen_open" \
  $P/crates/horizon-client/src/game/native_hooks.rs
grep -n "fn native_item_energy\b\|fn native_item_energy_extract" \
  $P/crates/horizon-client/src/game/native_item_energy.rs
```

## Not in scope

No `.hmod` rebuild, no `stage/check/test/e2e`, no `compatibility.json` edit. When the slices are
implemented, follow the campaign build hygiene (`CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=2`) and the
`stage → check → test → build` ladder.
