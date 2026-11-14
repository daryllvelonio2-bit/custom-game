# custom-game

A game built on [`custom-game-engine`](../custom-game-engine), which lives in its own
repository. Engine and game are separate projects, as required.

## Design goal

The full game must run in **<=100 MB RAM** on low-end hardware. The engine tracks RSS
per frame and enforces a hard memory budget.

## Run

```bash
cargo run
```

## Verification

```bash
cargo check
cargo run   # observe peak RSS via the engine's per-frame memory report
```
