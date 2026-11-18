#!/usr/bin/env bash
# In-game check for Construction Sticks slice 1 (plane construction).
#
# Three phases, all in survival so the supply is actually paid:
#
#   1. stone wall 5x3: a wooden stick (limit 3) right-clicked on its south face
#      must place exactly 3 stone in the face's plane (z=1) and consume
#      exactly 3 stone items.
#   2. stone-brick stairs wall: the three placed stairs must carry the clicked
#      stair's `facing=north`. The stairs' default state is `facing=east` and the
#      SDK has no `getStateForPlacement` query, so the glue copies the clicked
#      state: a default-state write would leave facing=east and fail here.
#   3. stone wall with a replaceable cell (a snow layer) in the plane: the fill
#      replaces it, as upstream's `replace` option (default on) does. A cell may
#      only be air in this port's 0.1.0, so the 0.1.0 rule would leave the snow
#      and place its three blocks elsewhere.
#
#   GAME=<launcher-built horizon-client> tools/e2e.sh
#   GAME=… COUNTERFACTUAL=1 tools/e2e.sh    # same click, plain minecraft:stick:
#                                          # 0 blocks placed, 0 stone consumed
#   GAME=… SELFTEST=1 tools/e2e.sh          # expects four placed blocks: must FAIL
#   GAME=… CONTROL_GAME=<a game without this package> tools/e2e.sh
#                                          # the control: the stick ids are unknown
#
# The counterfactual is how the check proves it can fail: the scenario and every
# assertion are unchanged, only the port's item is withheld. A plain stick has no
# use, so it must place nothing and consume nothing. SELFTEST is the same proof on
# the default run (an assertion that must not hold).
#
# The SDK package is one flat `.hmod` that carries both halves (code in `src/`, the
# sticks' assets/data at the root), so the game must be built **by the launcher**
# with this package installed — there is no `--dev-mod` content folder and no
# embedded-content helper any more:
#
#   horizon-launcher native-build <this port> --instance <instance> --trust-native-code
#
# then point GAME at that instance's `horizon-client`. `CONTROL_GAME` is an
# optional second instance built **without** this package; when it is given, the run
# first proves the stick cannot be given there (the phases cannot pass vacuously).
#
# NOT RUN in the 0.2.0 migration: `horizon-launcher` is not installed in this
# environment (see notes.md, "Not re-run in this migration"). The 0.1.0 transcripts
# remain the behaviour's evidence; this script is what must be re-run to claim
# `ported`.
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/constructionstick
# The control binary is shared at the campaign pin; the game is the launcher's build.
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/constructionstick-e2e-run.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
SELFTEST=${SELFTEST:-0}
WORLD=${WORLD:-ConstructionStickE2E}

[ -n "$GAME" ] || {
    echo "GAME must name a launcher-built horizon-client carrying constructionstick.hmod" >&2
    echo "  horizon-launcher native-build $PORT --instance <i> --trust-native-code" >&2
    exit 1
}
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/horizon-mod.toml" ] || { echo "missing the SDK package: $PORT" >&2; exit 1; }

if [ "$COUNTERFACTUAL" = 1 ]; then
    TOOL=minecraft:stick
    EXPECT_PLACED=0
    EXPECT_CONSUMED=0
else
    TOOL=constructionstick:wooden_stick
    EXPECT_PLACED=3
    EXPECT_CONSUMED=3
fi
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting 4 blocks placed instead of $EXPECT_PLACED)"
    EXPECT_PLACED=4
fi

SCRATCH=$(mktemp -d /tmp/opencode/constructionstick-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
# The game's game folder defaults to the working directory: run from scratch so
# saves/options/caches never land in the port tree.
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
count_item() { ctl inventory | jq -r --arg i "$1" '[.counts[$i] // 0] | add'; }
# The built plane, exactly: the 5x3 face at z=1 (x -3..3, y 100..102). The
# wall itself is at z=0, so every stone in this box is a placed cell.
plane_stone() { ctl blocks -3 100 1 3 102 1 | jq '[.blocks[] | select(.[3] == "minecraft:stone")] | length'; }
plane_stairs() { ctl blocks -3 100 1 3 102 1 | jq '[.blocks[] | select(.[3] | startswith("minecraft:stone_brick_stairs"))] | length'; }
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // "unknown"'; }
block_id() { ctl block "$1" "$2" "$3" | jq -r '.block // "unknown"'; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

# The player must be where the assertions think it is: `ctl tp` sets the client
# position and the server corrects it ("moved wrongly"), so the server-side
# teleport is used and its result checked.
place_player() {
    ctl command "tp @s 0 100 2" >/dev/null
    ctl wait 2
    local pos
    pos=$(ctl state | jq -r '.player.position | @csv')
    echo "player: $pos"
    case "$pos" in
        0.5,100*,2.5* | 0,100*,2*) ;;
        *) fail "the player is at $pos, not at the wall" ;;
    esac
}
# Clear the plane, rebuild the wall of $1 at z=0, and arm the player.
rebuild() {
    ctl command "kill @e[type=!minecraft:player,type=!minecraft:item]" >/dev/null
    ctl fill -6 100 -6 6 104 6 minecraft:air >/dev/null
    ctl fill -2 100 0 2 102 0 "$1" >/dev/null
    ctl command "clear @s" >/dev/null
    # The stick pays with the clicked block's own item.
    ctl give "$1" 64 >/dev/null || fail "cannot give $1"
    ctl give "$TOOL" 1 >/dev/null || fail "the content mod did not provide $TOOL"
    ctl wait 2
    ctl select "$TOOL" >/dev/null || fail "cannot select $TOOL"
    ctl wait 2
    local held
    held=$(ctl inventory | jq -r '.held.item // "none"')
    echo "held: $held"
    [ "$held" = "$TOOL" ] || fail "the held item is $held, expected $TOOL"
}
# Aim at the wall's south face and right-click it.
click_wall() {
    ctl look-at-block 0 101 0 south || fail "cannot aim at the wall's south face"
    ctl wait 2
    ctl use-item
    ctl wait 10
}

launch() { # launch <game binary> <world>
    rm -f "$SOCK"
    "$CTL" --socket "$SOCK" launch \
        --game "$1" --world "$2" --seed 20261102 --headless --wait \
        --game-dir "$SCRATCH/game" --resources "$RES" >/dev/null
    ctl wait-ready 120
}

# CONTROL_GAME: a build without this package. The stick ids must be unknown there,
# so the phases above cannot pass on a game that lacks the mod.
control_leg() {
    echo "== control: a game built without this package =="
    launch "$CONTROL_GAME" ConstructionStickControl
    ctl command "clear @s" >/dev/null
    ctl give constructionstick:wooden_stick 1 >/dev/null 2>&1 || true
    ctl wait 3
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "constructionstick:wooden_stick")] | length == 0' >/dev/null \
        || fail "the control game has the construction stick, so it is not a build without this package"
    echo "  without the package: no construction stick could be given"
    ctl quit >/dev/null 2>&1 || true
}

if [ -n "$CONTROL_GAME" ]; then
    [ -x "$CONTROL_GAME" ] || { echo "missing CONTROL_GAME: $CONTROL_GAME" >&2; exit 1; }
    control_leg
fi

echo "== launch (survival world $WORLD, one .hmod carrying content and code) =="
launch "$GAME" "$WORLD"

echo "== a stone platform in open air (y=99), wall 5x3 at z=0 and y=100..102 =="
ctl command "time set noon"
ctl command "difficulty peaceful"
ctl fill -6 99 -6 6 99 6 minecraft:stone
ctl wait 2
place_player

echo "== phase 1: stone wall, wooden stick (limit 3) =="
rebuild minecraft:stone
before=$(count_item minecraft:stone)
plane_before=$(plane_stone)
echo "stone before: $before"
echo "plane stone before: $plane_before"
click_wall
after=$(count_item minecraft:stone)
plane_after=$(plane_stone)
echo "stone after: $after"
echo "plane stone after: $plane_after"
ctl blocks -3 100 1 3 102 1 | jq -c '{counts, blocks}'

[ "$before" -ge "$EXPECT_CONSUMED" ] || fail "not enough stone to consume"
[ $((plane_after - plane_before)) -eq "$EXPECT_PLACED" ] \
    || fail "the click placed $((plane_after - plane_before)) blocks, expected $EXPECT_PLACED"
[ $((before - after)) -eq "$EXPECT_CONSUMED" ] \
    || fail "the click consumed $((before - after)) stone, expected $EXPECT_CONSUMED"

if [ "$COUNTERFACTUAL" = 0 ]; then
    echo "== phase 2: the placed stairs carry the clicked facing (default is facing=east) =="
    rebuild minecraft:stone_brick_stairs
    click_wall
    stairs=$(plane_stairs)
    echo "plane stairs after: $stairs"
    [ "$stairs" -eq 3 ] || fail "the click placed $stairs stairs, expected 3"
    for cell in "0 101 1" "1 101 1" "-1 101 1"; do
        # shellcheck disable=SC2086
        state=$(block_state $cell)
        echo "stairs at $cell: $state"
        case "$state" in
            *"facing=north"*) ;;
            *) fail "the stair at $cell is $state, not facing=north (a default-state write leaves facing=east)" ;;
        esac
    done

    echo "== phase 3: a replaceable cell (snow layer) is filled instead of skipped =="
    rebuild minecraft:stone
    ctl setblock 0 101 1 minecraft:snow >/dev/null
    ctl wait 2
    [ "$(block_id 0 101 1)" = "minecraft:snow" ] || fail "the replaceable cell is not snow before the click"
    before=$(count_item minecraft:stone)
    click_wall
    after=$(count_item minecraft:stone)
    replaced=$(block_id 0 101 1)
    placed=$(plane_stone)
    echo "replaceable cell after: $replaced"
    echo "plane stone after: $placed"
    echo "stone after: $after"
    [ "$replaced" = "minecraft:stone" ] \
        || fail "the click left $replaced in the replaceable cell: replaceable cells were skipped"
    [ "$placed" -eq 3 ] || fail "the click placed $placed blocks, expected 3"
    [ $((before - after)) -eq "$placed" ] \
        || fail "the click consumed $((before - after)) stone for $placed placed blocks"
fi

ctl quit >/dev/null 2>&1 || true
trap - EXIT
sleep 1
rm -rf "$SCRATCH" || true
if [ "$COUNTERFACTUAL" = 1 ]; then
    echo "PASS (counterfactual, plain minecraft:stick): 0 blocks placed, 0 stone consumed"
else
    echo "PASS: the wooden stick filled 3 blocks in the clicked face's plane and consumed 3 stone; the stairs carried the clicked facing=north; a replaceable snow cell was filled"
fi
echo "EXIT:0"
