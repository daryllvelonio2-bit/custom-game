# custom-game

A game built on [`custom-game-engine`](../custom-game-engine), which lives in its own
repository. Engine and game are separate projects, as required.

## Design goal

The full game must run in **<=100 MB RAM** on low-end hardware. The engine tracks RSS
per frame and enforces a hard memory budget.

## Layout

| Path | Contents |
|---|---|
| `src/lib.rs` | Game logic. No GPU, no window, no audio — tests here run in seconds. |
| `src/main.rs` | Thin `App` wiring: builds the window and forwards frames to `lib.rs`. |

The engine is an **optional dependency**, enabled by the default `app` feature that the
binary requires. That is what keeps `cargo t` / `cargo l` from rebuilding `wgpu`.

## Fast dev loop (seconds)

| Command | What it does | Typical |
|---|---|---|
| `cargo t` | test game logic only (no GPU stack) | ~1 s |
| `cargo q` | check game logic only | ~1 s |
| `cargo l` | clippy game logic only | ~1 s |
| `cargo c` | check everything incl. engine | ~3 s warm |
| `cargo r` | run: fast compile, slower frame rate | ~2 s relink |
| `cargo r-play` | run on `dev-opt`: near-release frame rate | slower to compile |
| `cargo b` | normal dev build | — |
| `cargo build --release` | shipping build (`lto`, 1 CGU) | minutes, expected |

Only the release build is slow on purpose. Everything else is incremental and shares one
target dir (`Documents/GitHub/custom-target`) with the engine repo, so no dependency is
compiled twice. `../dev.sh` runs the whole gate (engine + game) in one command.

## Run

```bash
cargo run
```

## Verification

```bash
cargo c            # check, including the engine
cargo t            # game logic tests
../dev.sh          # both repos: check + test + clippy
cargo run          # observe peak RSS via the engine's per-frame memory report
```
