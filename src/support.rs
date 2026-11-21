//! Game-side glue for Construction Sticks slice 1 (the plane fill).
//!
//! [`crate::runtime`] decides which cells one use fills and every number it uses;
//! this module is the only place that reads the game and writes it back: the clicked
//! block and its state, the supply in the player's inventory, the blocks themselves,
//! and the wear of the held stick.
//!
//! The 0.1.0 bundle did the same through the retired native hooks; the mapping is in
//! `notes.md` under `## Migration to the SDK format`.

use mod_api::{
    BlockFace, BlockState, BlockUpdateFlags, Hand, HostBlockStates, HostSide, InteractionResult,
    ItemKey, ItemPart, ItemStack, LevelRef, PlayerRef, Registrar, ServerHost, UseOnEvent,
};

use crate::runtime;

/// The five sticks the content declares, in tier order.
const STICK_IDS: [&str; 5] = [
    runtime::WOODEN_STICK,
    runtime::COPPER_STICK,
    runtime::IRON_STICK,
    runtime::DIAMOND_STICK,
    runtime::NETHERITE_STICK,
];

/// The main inventory's slots: the hotbar (0 to 8) and then the 27 backpack slots.
/// Equipment and the off hand are not supply, as upstream pays from the main
/// inventory first.
pub const MAIN_INVENTORY_END: u32 = 36;

/// `player_set_inventory_slot`'s number for the off hand.
pub const OFF_HAND_SLOT: u32 = 40;

/// The inventory slot of the hand a stick was used from: the main hand is the
/// player's selected hotbar slot, the off hand is its own.
pub fn hand_slot(hand: Hand, selected: u32) -> u32 {
    match hand {
        Hand::Main => selected,
        Hand::Off => OFF_HAND_SLOT,
    }
}

/// The damage a stack carries after one more placed block: `Some(next)` while it
/// lasts, `None` when this use breaks it. Vanilla breaks a stack when its damage
/// reaches the maximum, and an item with no durability (`max` of 0) never wears.
pub fn worn(damage: u32, max: u32) -> Option<u32> {
    if max == 0 {
        return Some(damage);
    }
    let next = damage.saturating_add(1);
    (next < max).then_some(next)
}

/// The world the planner reads: the block at a position by id, and whether a cell
/// may be built in.
///
/// A cell may be built in when it is replaceable and loaded (`block_state` answers
/// `None` outside a loaded chunk or the world's height). The 0.1.0 port also got
/// vanilla's living-entity check and `canSurvive` from the game's own placement path;
/// the SDK has no living-entity query (`level_entities_in` hands back `EntityRef`s and
/// nothing turns one into a `LivingRef`), so both are gaps in `notes.md`.
struct LevelWorld<'a> {
    host: &'a mut dyn ServerHost,
    level: LevelRef,
}

impl runtime::World for LevelWorld<'_> {
    fn block(&mut self, pos: [i32; 3]) -> Option<String> {
        let host: &dyn mod_api::Host = &*self.host;
        self.level.block(host, pos).map(|block| block.id(host))
    }

    fn can_place(&mut self, pos: [i32; 3]) -> bool {
        let Some(existing) = self.level.block_state(self.host, pos) else {
            return false;
        };
        self.host.block_states().replaceable(existing) == Some(true)
    }
}

/// How many items of `item` the main inventory holds.
fn count_items(host: &dyn ServerHost, player: PlayerRef, item: ItemKey) -> u32 {
    player
        .inventory_slots(host)
        .map(|slots| {
            slots
                .into_iter()
                .filter(|(slot, stack)| *slot < MAIN_INVENTORY_END && stack.item() == item)
                .map(|(_, stack)| stack.count())
                .sum()
        })
        .unwrap_or(0)
}

/// Take up to `count` items of `item` from the main inventory, returning how many
/// left it. A slot the game refuses to write stops the take where it is.
fn take_items(host: &mut dyn ServerHost, player: PlayerRef, item: ItemKey, count: u32) -> u32 {
    let Some(slots) = player.inventory_slots(host) else {
        return 0;
    };
    let mut wanted = count;
    let mut taken = 0;
    for (slot, stack) in slots {
        if wanted == 0 {
            break;
        }
        if slot >= MAIN_INVENTORY_END || stack.item() != item {
            continue;
        }
        let take = stack.count().min(wanted);
        let left = stack.count() - take;
        let next = if left == 0 {
            None
        } else {
            let mut next = stack;
            if next.set_count(left).is_err() {
                break;
            }
            Some(next)
        };
        if player.set_inventory_slot(host, slot, next).is_err() {
            break;
        }
        taken += take;
        wanted -= take;
    }
    taken
}

/// Take one durability off the stick after a placed block: `Some(next)` is the damage
/// it now carries (or the unchanged `damage` for an item without durability, which is
/// what the carried content ships), `None` means it broke and the fill stops.
fn wear(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    hand: Hand,
    stack: &ItemStack,
    damage: u32,
) -> Option<u32> {
    let Some(max) = host.max_damage(stack.item()) else {
        return Some(damage);
    };
    if max == 0 {
        return Some(damage);
    }
    let selected = u32::try_from(player.selected_slot(host)?).ok()?;
    let slot = hand_slot(hand, selected);
    match worn(damage, max) {
        Some(next) => {
            let mut worn_stack = stack.clone();
            worn_stack.set_damage(next);
            player
                .set_inventory_slot(host, slot, Some(worn_stack))
                .ok()?;
            Some(next)
        }
        None => {
            player.set_inventory_slot(host, slot, None).ok()?;
            None
        }
    }
}

/// What one use places and pays: the state copied into every cell, the item each
/// placed block costs, and the stick that wears.
struct Placement<'a> {
    state: BlockState,
    item: ItemKey,
    face: BlockFace,
    hand: Hand,
    stack: &'a ItemStack,
}

/// The `BlockFace` a click normal names.
fn block_face(normal: [i32; 3]) -> Option<BlockFace> {
    Some(match normal {
        [0, -1, 0] => BlockFace::Down,
        [0, 1, 0] => BlockFace::Up,
        [0, 0, -1] => BlockFace::North,
        [0, 0, 1] => BlockFace::South,
        [-1, 0, 0] => BlockFace::West,
        [1, 0, 0] => BlockFace::East,
        _ => return None,
    })
}

/// Whether `cell` lies inside the level's world border (the `[min_x, min_z, max_x,
/// max_z]` the host reports; no border means everything is inside). The direct
/// placement path checks loaded and absolute world bounds, not the border this
/// server configures.
fn in_border(border: Option<[f64; 4]>, cell: [i32; 3]) -> bool {
    border.is_none_or(|[min_x, min_z, max_x, max_z]| {
        let x = f64::from(cell[0]) + 0.5;
        let z = f64::from(cell[2]) + 0.5;
        x >= min_x && x <= max_x && z >= min_z && z <= max_z
    })
}

/// Whether the player may write at `cell`: the level's world border, and the game's
/// own `mayUseItemAt` for the held stick (build permission, and adventure's
/// `can_place_on`). Spawn protection is not exposed by this pin (`surface.json` has
/// no spawn-protection symbol), so it is recorded as an API gap in `notes.md`.
fn may_write(
    host: &dyn ServerHost,
    player: PlayerRef,
    level: LevelRef,
    cell: [i32; 3],
    face: BlockFace,
    stack: &ItemStack,
) -> bool {
    in_border(level.world_border(host), cell)
        && player.may_use_item_at(host, cell, face, stack) != Some(false)
}

/// Place the planned cells, paying one item of the block's kind and one durability
/// per placed block, as the 0.1.0 port did: creative places for free, a cell the game
/// refuses is skipped and the fill continues, and the last block is put back when the
/// supply runs out mid-run.
fn place(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    level: LevelRef,
    cells: &[[i32; 3]],
    placement: &Placement<'_>,
) {
    let Placement {
        state,
        item,
        face,
        hand,
        stack,
    } = *placement;
    let creative = player.is_creative(host).unwrap_or(false);
    let mut budget = if creative {
        u32::MAX
    } else {
        count_items(host, player, item)
    };
    let mut damage = stack.damage();
    for cell in cells {
        if budget == 0 {
            break;
        }
        // The direct placement path tests loaded chunks and the absolute world
        // bounds; it does not test the configured border or the game's own use
        // permission, so the fill checks both before writing a cell. A refused cell
        // is skipped, as a refusal from the old placement call was.
        if !may_write(&*host, player, level, *cell, face, stack) {
            continue;
        }
        // The closest supported shape of vanilla's placement path: the state is
        // written in the planned cell and `setPlacedBy` runs in the player's turn.
        if player
            .place_block(host, level, *cell, state, false)
            .is_err()
        {
            continue;
        }
        if !creative {
            if take_items(host, player, item, 1) == 0 {
                if let Some(air) = host.block_state_id("minecraft:air") {
                    let flags = BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS;
                    let _ = level.set_block(host, *cell, air, flags);
                }
                break;
            }
            budget -= 1;
            // Creative pays no durability (upstream skips the wear entirely), so a
            // creative stick never wears out however many blocks it places.
            match wear(host, player, hand, stack, damage) {
                Some(next) => damage = next,
                // The stick broke on the block just placed: this use is over.
                None => break,
            }
        }
    }
}

/// A right click with a construction stick on a block: fill the clicked face's plane
/// with copies of the clicked block. Answering consumes the click.
fn use_on_block(
    host: &mut dyn ServerHost,
    event: &mut UseOnEvent<'_>,
) -> Option<InteractionResult> {
    let tier = runtime::stick_for(&event.stack.item().id(host))?;
    let player = host.player(event.player)?;
    if player.is_spectator(host) == Some(true) {
        return None;
    }
    let level = player.level(host)?;
    let clicked = level.block(host, event.pos)?;
    let clicked_id = clicked.id(host);
    let state = level.block_state(host, event.pos)?;
    let item = host.block_item(clicked)?;
    if item.id(host) == "minecraft:air" {
        return None;
    }
    let creative = player.is_creative(host).unwrap_or(false);
    // The stick's own remaining durability caps every job, and in survival the supply
    // caps it too (creative pays nothing). Both are upstream's `StickJob` rules.
    let remaining =
        runtime::remaining_durability(host.max_damage(event.stack.item()), event.stack.damage());
    let supply = if creative {
        u32::MAX
    } else {
        count_items(host, player, item)
    };
    let payment = if creative {
        runtime::Payment::Creative
    } else {
        runtime::Payment::Survival
    };
    let limit = runtime::use_limit(tier.limit, payment, remaining, supply);
    if limit == 0 {
        return None;
    }
    let cells = {
        let mut world = LevelWorld { host, level };
        runtime::plan(&mut world, event.pos, event.face, &clicked_id, limit)
    };
    if cells.is_empty() {
        return None;
    }
    let face = block_face(event.face)?;
    place(
        host,
        player,
        level,
        &cells,
        &Placement {
            state,
            item,
            face,
            hand: event.hand,
            stack: event.stack,
        },
    );
    Some(InteractionResult::SuccessServer)
}

/// The stick behaviour, attached to the five stick items in `Hooks::register`. Being a
/// Part, the game asks it only for those ids, so the hook never checks the item.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stick;

#[mod_api::horizon_part]
impl ItemPart for Stick {
    /// `ItemStack.useOn`: the stick's right click on a block, which runs once the
    /// block itself let the click through. Answering consumes it; the client half
    /// answers nothing, so its prediction never writes the world.
    fn use_on(&self, host: HostSide<'_>, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        let HostSide::Server(host) = host else {
            return None;
        };
        use_on_block(host, event)
    }
}

/// Attach the stick behaviour to the five stick items the content declares.
pub fn register_sticks(registrar: &mut dyn Registrar) {
    for id in STICK_IDS {
        // `item_part(registrar.item_key(..), ..)` would be two `&mut` borrows of the
        // registrar at once (imfast migration trap 15): bind the key first.
        let key = registrar.item_key(id);
        registrar.item_part(key, Stick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hand_slot_follows_the_hand() {
        assert_eq!(hand_slot(Hand::Main, 4), 4);
        assert_eq!(hand_slot(Hand::Off, 4), OFF_HAND_SLOT);
    }

    #[test]
    fn a_durable_stack_wears_one_per_placed_block() {
        assert_eq!(worn(0, 59), Some(1));
        assert_eq!(worn(57, 59), Some(58));
    }

    #[test]
    fn the_last_use_breaks_a_durable_stack() {
        // Vanilla breaks an item when its damage reaches the maximum, so the use
        // that would make damage == max takes the stack with it.
        assert_eq!(worn(58, 59), None);
    }

    #[test]
    fn an_item_without_durability_never_wears() {
        assert_eq!(worn(0, 0), Some(0));
    }

    #[test]
    fn a_click_normal_names_the_face_the_game_checks() {
        assert_eq!(block_face([0, 1, 0]), Some(BlockFace::Up));
        assert_eq!(block_face([0, 0, 1]), Some(BlockFace::South));
        assert_eq!(block_face([3, 0, 0]), None);
    }

    #[test]
    fn a_cell_outside_the_world_border_is_not_written() {
        // [min_x, min_z, max_x, max_z] as level_world_border reports it.
        let border = Some([-8.0, -8.0, 8.0, 8.0]);
        assert!(in_border(border, [0, 64, 0]));
        assert!(in_border(border, [7, 64, -7]));
        assert!(!in_border(border, [9, 64, 0]), "east of the border");
        assert!(!in_border(border, [0, 64, -9]), "north of the border");
        assert!(
            in_border(None, [1_000_000, 64, 1_000_000]),
            "no border, no refusal"
        );
    }

    #[test]
    fn every_stick_the_content_declares_is_registered() {
        // The five ids the content ships are the five the Part answers for: a stick
        // left out here is an item the player can craft and that does nothing.
        for id in STICK_IDS {
            assert!(runtime::stick_for(id).is_some(), "{id} has no tier");
        }
        assert_eq!(STICK_IDS.len(), 5);
    }

    /// The five item declarations as packaged. The file the game loads is the file
    /// read here, so the tier table in [`runtime`] and the carried durability cannot
    /// drift apart.
    const ITEM_FILES: [(&str, &str); 5] = [
        (
            runtime::WOODEN_STICK,
            include_str!("../data/constructionstick/horizon/item/wooden_stick.json"),
        ),
        (
            runtime::COPPER_STICK,
            include_str!("../data/constructionstick/horizon/item/copper_stick.json"),
        ),
        (
            runtime::IRON_STICK,
            include_str!("../data/constructionstick/horizon/item/iron_stick.json"),
        ),
        (
            runtime::DIAMOND_STICK,
            include_str!("../data/constructionstick/horizon/item/diamond_stick.json"),
        ),
        (
            runtime::NETHERITE_STICK,
            include_str!("../data/constructionstick/horizon/item/netherite_stick.json"),
        ),
    ];

    /// The number a `"field": <n>` pair holds in one declaration. The declarations
    /// have the fixed shape `content_declarations::Item` reads, and the crate has no
    /// JSON dependency, so this scans for the field and the integer after it.
    fn number_field(json: &str, field: &str) -> Option<u32> {
        let after = &json[json.find(field)? + field.len()..];
        let start = after.find(|c: char| c.is_ascii_digit())?;
        let digits: String = after[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    }

    #[test]
    fn every_stick_declares_its_tier_durability_and_does_not_stack() {
        for (id, json) in ITEM_FILES {
            let tier = runtime::stick_for(id).expect("every packaged stick has a tier");
            assert_eq!(
                number_field(json, "max_stack_size"),
                Some(1),
                "{id}: upstream's sticks are one per stack"
            );
            assert_eq!(
                number_field(json, "max_damage"),
                Some(tier.durability),
                "{id}: the carried durability is its tier's"
            );
        }
    }
}
