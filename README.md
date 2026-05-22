# hawxide-rs

`hawxide-rs` is a cross-platform terminal runner game written in Rust with
[Ratatui](https://ratatui.rs/). The game is inspired by the offline Chrome
Dinosaur game, but the player is a hawk navigating three vertical lanes of
Oxide rack obstacles.

The display uses ASCII characters only. Sprite masks are shared by rendering
and collision logic so game over happens only when visible player cells overlap
visible obstacle cells.

## Gameplay

- The hawk starts in the middle lane.
- Press `Up` or `e` to move one lane higher.
- Press `Down` or `d` to move one lane lower.
- The hawk stays in its selected lane until moved again.
- Press `q` to quit.
- After a collision, press `r` to restart or `q` to quit.

The lanes are hard boundaries. Repeated upward input in the high lane keeps the
hawk in the high lane, and repeated downward input in the low lane keeps the
hawk in the low lane.

## Obstacles

Three obstacle types scroll from right to left:

- Low lane: a horizontal Oxide rack lying on its side.
- Middle/low band: a truck carrying an Oxide rack.
- High/middle band: an Oxide rack dropped by parachute.

The low rack is confined to the low lane. The truck may occupy the combined
middle and low lanes. The parachute rack may occupy the combined high and
middle lanes. Obstacle spawning preserves enough horizontal space for the
player to step between lanes, so the game should not create impossible
sequences.

## Development

Install Rust with `rustup`, then run:

```sh
cargo run
```

Useful checks:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Project Layout

- `src/main.rs` owns terminal setup, input polling, and cleanup.
- `src/game.rs` owns lane movement, scoring, obstacle spawning, sprite masks,
  and collision.
- `src/render.rs` owns Ratatui drawing.
