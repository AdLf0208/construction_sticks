# Validation

Validated on Linux with Rust 1.95.0 on 2026-10-30, using the exact revisions in
`compatibility.json` and the real bundle committed here.

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Pass |
| `cargo test --workspace --locked --offline` | Runtime unit test passes |
| `python3 tools/test_template.py` | Actual bundle compiles in the tiny fixture; original method calls/result, shared runtime, AST audit, unchanged bytes/mtimes and incompatible revision/target rejection pass |
| `python3 tools/test_stage.py` | Complete pinned Horizon export stages deterministically; original checkout unchanged; exact HEAD/module audit passes |
| `python3 tools/horizon.py check` | Actual `horizon-client` binary checks successfully with `--no-default-features` |
| `python3 tools/horizon.py test` | The runtime's embedded test passes inside the actual Horizon game test binary |

The native check reported 69.55 seconds in Cargo, and the native test passed.
Both reused an existing external Cargo target directory with cached
Horizon dependencies. These are compilation/test checks, not a fresh-build
benchmark. Staging the full export took about 24 seconds per command on this
machine; a repeated application preserved the stage.

No game was launched and no Minecraft resources or player data were read or
changed. Interactive logging/gameplay remains a manual check with the compatible
prepared resources. The source audit verifies only the declared HEAD statement
and support module differ from pristine Rust ASTs; it does not claim compatibility
with other Horizon revisions or arbitrary third-party mods.

CI reproduces the formatting, clippy, standalone runtime and tiny-fixture tests.
The full native checks additionally need a Horizon Git history containing the
pin and its cached dependencies; use the commands in README.md to reproduce them.

## Campaign pin `1d63e0a5` (2026-11-10)

Migrated from `83ad384f` / mixins `e8cd0aa` to Horizon
`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` (mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`, schema 6), with the content mod
carried inside the one `.hnmod` (`tools/package.sh` builds the `.hmod`, copies it
to `mixins/constructionstick/data/constructionstick-content.hmod`, then
`tools/package.py` writes the bundle). All steps ran through
`ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`.

| Check | Result |
|---|---|
| `cargo fmt -- --check` | pass |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | pass, 0 warnings |
| `cargo test --locked --offline` | 8 passed, 0 failed |
| `tools/horizon.py stage/check/test/build` at the pin | `cargo_exit: 0`, 0 warnings, 8 embedded tests, fast build |
| `EMBEDDED=1 tools/e2e.sh` (control + 3 phases, no `--dev-mod`) | **PASS**, `EXIT:0` |
| `SELFTEST=1 tools/e2e.sh` | **FAIL** (asserts 4 blocks, 3 placed), `EXIT:1` |
| `content/constructionstick.hmod` | 86,208 B, SHA-256 `3f1cd6f3…596d3f`, 157 entries |
| `constructionstick.hnmod` | 62,907 B, SHA-256 `a9dd66b2…fd69f94`, 7 entries incl. the embedded content |

The control leg runs the same binary with the game folder's `.horizon-cache`
made unusable: the stick cannot be given, so the item proved in the main leg
comes from the embedded content file (written byte-identical to
`content/constructionstick.hmod`).

## 0.2.0 — the SDK package (2026-11-18)

The 0.1.0 ladder above is retired with the `.hnmod` format (`tools/horizon.py`,
`tools/test_stage.py` and `tools/test_template.py` are gone). The SDK ladder, run in a
`build-slot.sh` lease directly in the port:

| Check | Result |
|---|---|
| `cmp` + `sha256sum` of `src/runtime.rs` against the retired file, and of the packaged entry | byte-identical, `3acc74b1…e5230` |
| `CARGO_INCREMENTAL=0 cargo test --offline` (before any glue) | 8 passed, 0 failed |
| `cargo test --offline` | **13 passed**, 0 failed |
| `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings |
| `cargo fmt --check` | clean |
| `rm -f Cargo.lock && cargo horizon-mod build` | exit 0, `constructionstick.hmod` |
| a second and a third build (one from a deleted `target/`) | same 102,127 B, same sha256 `0660f04c…5136` |
| `cargo horizon-mod check constructionstick.hmod` | exit 0 |
| `cargo horizon-mod fits constructionstick.hmod --build nightly` | `0b14f8a  made for`, exit 0 |
| in-game `tools/e2e.sh` | **not run** — `horizon-launcher` is not installed here, so no launcher-built game exists |

Logs: `ports/_shared/evidence/constructionstick/sdk-*.log`. The 0.1.0 in-game transcripts
are historical (see that folder's `README.md`).
