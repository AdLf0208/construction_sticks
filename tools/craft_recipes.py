#!/usr/bin/env python3
"""Craft every carried Construction Sticks recipe in a real crafting table.

Called by `tools/e2e.sh` with the open game's control socket and the position of
a placed crafting table. The harness's table screen is a 3x3 grid at slots
100..108 (row-major) with the result at 200; the player's own slots keep their
inventory numbers (the hotbar is 0..8).

What it proves, for each of the ten recipes the package carries:

1. the pattern the recipe declares makes exactly one of its result item — the
   ids are asserted, so a recipe that loses its result fails here;
2. every ingredient is consumed (its inventory count is 0 afterwards) and the
   grid is empty;
3. the crafted item is in the inventory and can be read back by id.

`SELFTEST=recipe` expects the second stick's id for the first recipe, so the run
MUST exit 1; it is the proof this assertion can fail.

Exit 1 with the observed result on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

SELFTEST = os.environ.get("SELFTEST") == "recipe"

# The table's grid, row-major, and its result.
GRID = {
    (0, 0): 100, (0, 1): 101, (0, 2): 102,
    (1, 0): 103, (1, 1): 104, (1, 2): 105,
    (2, 0): 106, (2, 1): 107, (2, 2): 108,
}
RESULT = 200
HOTBAR = 0

# (result id, pattern, key char -> item id) for the ten carried recipes, copied
# from `data/constructionstick/recipe/*.json`; `#` is `#c:rods/wooden` (a stick)
# in the stick recipes and `#c:glass_blocks` (glass) in the template recipes.
# The four metal sticks put the tip in the centre: the jar's diagonal tip is
# 26.3's spear shape, and vanilla's spear recipe won the grid (see notes.md).
RECIPES = [
    ("wooden_stick", ["  X", " # ", "#  "], {"X": "minecraft:stick", "#": "minecraft:stick"}),
    ("copper_stick", ["  #", " X ", "#  "], {"X": "minecraft:copper_ingot", "#": "minecraft:stick"}),
    ("iron_stick", ["  #", " X ", "#  "], {"X": "minecraft:iron_ingot", "#": "minecraft:stick"}),
    ("diamond_stick", ["  #", " X ", "#  "], {"X": "minecraft:diamond", "#": "minecraft:stick"}),
    ("netherite_stick", ["  #", " X ", "#  "], {"X": "minecraft:netherite_ingot", "#": "minecraft:stick"}),
    ("template_angel", [" #X", "#O#", "X# "], {"#": "minecraft:glass", "X": "minecraft:gold_ingot", "O": "minecraft:feather"}),
    ("template_battery", [" #X", "#O#", "X# "], {"#": "minecraft:glass", "X": "minecraft:redstone", "O": "minecraft:potato"}),
    ("template_destruction", [" #X", "#O#", "X# "], {"#": "minecraft:glass", "X": "minecraft:diamond_pickaxe", "O": "minecraft:tnt"}),
    ("template_replacement", [" #X", "#O#", "X# "], {"#": "minecraft:glass", "X": "minecraft:sculk", "O": "minecraft:ender_pearl"}),
    ("template_unbreakable", [" #X", "#O#", "X# "], {"#": "minecraft:glass", "X": "minecraft:crying_obsidian", "O": "minecraft:nether_star"}),
]


def layout(pattern: list[str], key: dict[str, str]) -> tuple[list[tuple[int, str]], dict[str, int]]:
    """The (slot, item) placements and how many of each item the pattern needs."""
    slots: list[tuple[int, str]] = []
    need: dict[str, int] = {}
    for row, line in enumerate(pattern):
        for col, char in enumerate(line):
            if char == " ":
                continue
            item = key[char]
            slots.append((GRID[(row, col)], item))
            need[item] = need.get(item, 0) + 1
    return slots, need


def main() -> int:
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]
    x, y, z = int(x), int(y), int(z)

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"FAIL: no menu is open: {screen}")
        return screen["slots"]

    def item_of(slot: int) -> str | None:
        found = [s for s in slots() if s["slot"] == slot]
        return (found[0].get("item") if found else None) or None

    def count_of(item: str) -> int:
        return run("inventory")["counts"].get(item, 0)

    def find(item: str) -> int:
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"FAIL: missing ingredient {item} (inventory: {run('inventory')['counts']})")

    def result() -> dict | None:
        found = [s for s in slots() if s["slot"] == RESULT]
        if not found or not found[0].get("item"):
            return None
        return found[0]

    def place(item: str, targets: list[int]) -> None:
        """Put one `item` in each target, a stack at a time.

        One target per pick-up: an unstackable ingredient (a diamond pickaxe in
        the destruction template) arrives as several stacks of one, so a single
        pick-up could not fill two cells.
        """
        for target in targets:
            slot = find(item)
            run("click-slot", str(slot))
            wait(1)
            run("click-slot", str(target), "right")
            wait(1)
            run("click-slot", str(slot))
            wait(1)

    def open_table(need: dict[str, int]) -> None:
        try:
            run("close")
        except SystemExit:
            pass
        wait(4)
        run("command", "clear @s")
        for item, count in need.items():
            run("give", item, str(count))
        wait(3)
        run("interact", str(x), str(y), str(z))
        wait(4)
        if "slots" not in run("screen"):
            raise SystemExit("FAIL: the crafting table did not open")

    def take_result() -> None:
        run("click-slot", str(RESULT))
        wait(1)
        run("click-slot", str(HOTBAR))
        wait(2)

    for index, (name, pattern, key) in enumerate(RECIPES):
        result_id = f"constructionstick:{name}"
        placements, need = layout(pattern, key)
        if SELFTEST and index == 0:
            result_id = "constructionstick:copper_stick"
            print(f"(selftest: expecting {result_id} from the wooden-stick pattern)")
        open_table(need)
        for item in need:
            place(item, [slot for slot, placed in placements if placed == item])
        got = result()
        print(f"  {name}: {json.dumps(got, sort_keys=True)}")
        if not got or got.get("item") != result_id or got.get("count") != 1:
            raise SystemExit(f"FAIL: {name}: expected 1 {result_id}, got {got}")
        take_result()
        made = count_of(result_id)
        left = {item: count_of(item) for item in need}
        grid = [s for s in slots() if s["slot"] in GRID.values() and s.get("item")]
        print(f"    crafted {made} {result_id}, ingredients left {left}, grid {json.dumps(grid)}")
        if made != 1:
            raise SystemExit(f"FAIL: {name}: the craft left {made} {result_id}, expected 1")
        if any(left.values()):
            raise SystemExit(f"FAIL: {name}: ingredients not consumed: {left}")
        if grid:
            raise SystemExit(f"FAIL: {name}: the grid still holds {grid}")
    run("close")
    return 0


if __name__ == "__main__":
    sys.exit(main())
