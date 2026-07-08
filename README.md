# Rust Asteroids

A modern, vector-style rendition of the arcade classic, written in Rust with
[macroquad](https://macroquad.rs/). Pilot your ship through increasingly busy
asteroid fields, split rocks into smaller hazards, and chase the high score.

## Features

- Momentum-based ship controls and screen wrapping
- Large, medium, and small asteroids with procedural silhouettes
- Progressive waves that grow faster and more crowded
- Scoring, lives, respawn invulnerability, and session high score
- Particle effects, starfield, pause screen, and responsive letterboxing
- No external art assets required

## Controls

| Action | Keyboard |
| --- | --- |
| Turn | `A` / `D` or `←` / `→` |
| Thrust | `W` or `↑` |
| Fire | `Space` |
| Pause / resume | `P` or `Esc` |
| Start / restart | `Enter` or `Space` |

## Run locally

Install [Rust](https://www.rust-lang.org/tools/install), then:

```sh
git clone https://github.com/daverozee/rust-asteroids.git
cd rust-asteroids
cargo run --release
```

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## License

Released under the [MIT License](LICENSE).

> Asteroids is a trademark of Atari, Inc. This fan-made project is not
> affiliated with or endorsed by Atari.
