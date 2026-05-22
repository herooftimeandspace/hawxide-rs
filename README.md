# hawxide-rs

`hawxide-rs` is a cross-platform terminal runner game written in Rust with
[Ratatui](https://ratatui.rs/). The game is inspired by the offline Chrome
Dinosaur game, but the player is a hawk navigating three vertical lanes of
Oxide rack obstacles.

Gameplay sprites use ASCII characters so the collision masks are easy to read.
The surrounding terminal UI uses Ratatui's native widgets, including UTF-8
borders when the terminal supports them. Obstacle art may use UTF-8 box drawing
and block characters for denser Oxide rack shapes. Sprite masks are shared by
rendering and collision logic so game over happens only when visible player
cells overlap visible obstacle cells.

The hawk is drawn with ASCII characters and Ratatui colors intended to evoke a
red-tailed hawk: brown upper markings, pale body text, yellow accents, and red
tail markings.

## Gameplay

- The game opens on a `HAWXIDE` splash screen.
- Press any key to start from the splash screen.
- The hawk starts in the middle lane.
- Press `Up` or `e` to move one lane higher.
- Press `Down` or `d` to move one lane lower.
- The hawk stays in its selected lane until moved again.
- Press `q` to quit.
- After a collision, press `r` to restart or `q` to quit.

The lanes are invisible hard boundaries. Repeated upward input in the high lane
keeps the hawk in the high lane, and repeated downward input in the low lane
keeps the hawk in the low lane. The visible playfield is capped and centered in
large terminals so the obstacle spacing and art proportions remain playable.
Obstacle scroll speed starts increasing immediately and continues to ramp up
with survival time.

## Obstacles

Three obstacle types scroll from right to left:

- Low lane: a horizontal Oxide rack lying on its side.
- Middle/low band: a tall standing Oxide rack.
- High/middle band: an Oxide rack dropped by parachute.

The low rack is confined to the low lane. The tall rack may occupy the combined
middle and low lanes. The parachute rack may occupy the combined high and
middle lanes. Obstacle order is randomized during play, with immediate repeats
suppressed so the sequence does not feel like a fixed cycle. Obstacle spawning
preserves enough horizontal space for the player to step between lanes, and the
required spacing grows with scroll speed so faster runs still leave a reaction
window.

## Background

Clouds and trees are decorative background objects. Their sprite variants are
randomized during play. Clouds render in light blue. Trees render with brown
trunks and dark green leaves. They scroll more slowly than obstacles and never
participate in collision detection.

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
