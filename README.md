# custom-game

A game built on `custom-game-engine`, whose source is vendored in this repository at
`src/custom-engine/`. One checkout, one Cargo workspace, one lockfile.

## Design goal

The full game must run in **<=100 MB RAM** on low-end hardware. The engine tracks RSS
per frame and enforces a hard memory budget.

## Layout

| Path | Contents |
|---|---|
| `src/lib.rs` | Game logic. No GPU, no window, no audio — tests here run in seconds. |
| `src/main.rs` | Thin `App` wiring: builds the window and forwards frames to `lib.rs`. |
| `src/custom-engine/` | The engine crate (own manifest, workspace member). Engine crates only. |

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
target dir (`Documents/GitHub/custom-target`) through `.cargo/config.toml`, so no
dependency is compiled twice. `../dev.sh` runs the whole gate (engine + game) in one
command.

## Run

```bash
cargo run
```

## Verification

```bash
cargo c                      # check the game, including the vendored engine
cargo t                      # game logic tests
cargo test --workspace       # engine + game tests
../dev.sh                    # both repos: check + test + clippy
cargo run                    # observe peak RSS via the engine's per-frame memory report
```
