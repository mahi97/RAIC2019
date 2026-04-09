# Parsian RAIC 2019 — CodeBall Strategy (Rust)

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#building)
[![Language](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![Competition](https://img.shields.io/badge/competition-RAIC%202019-blue)](https://raic.io/)

> **Parsian** team's base strategy implementation for the [Russian AI Cup 2019 (CodeBall)](https://raic.io/) competition, written in Rust.

---

## Table of Contents

- [About the Competition](#about-the-competition)
- [Architecture Overview](#architecture-overview)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Building](#building)
  - [Native Build](#native-build)
  - [Docker Build](#docker-build)
- [Running](#running)
  - [Against the Local Server](#against-the-local-server)
  - [Using Helper Scripts](#using-helper-scripts)
- [Key Modules](#key-modules)
- [Contributing](#contributing)

---

## About the Competition

**RAIC 2019 — CodeBall** is a 3-D robotic soccer game hosted by the Russian AI Cup.  
Two teams of robots compete inside a rounded arena to score the most goals by
hitting a ball into the opponent's net.  Each robot can:

- Move along the arena floor at up to `ROBOT_MAX_GROUND_SPEED`
- Jump to reach aerial balls (`ROBOT_MAX_JUMP_SPEED`)
- Collect nitro packs for an acceleration boost (`ROBOT_NITRO_ACCELERATION`)

The game server communicates with contestant programs over TCP using a JSON
protocol.  Each tick, the server sends the current `Game` state; the program
replies with an `Action` for every friendly robot.

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│                     main.rs                          │
│  • Parses CLI arguments (host, port, token)          │
│  • Connects to the game server via RemoteProcessClient│
│  • Game loop: read Game → call Strategy::act → write │
└───────────────────────┬──────────────────────────────┘
                        │
          ┌─────────────▼─────────────┐
          │       MyStrategy          │
          │  (src/my_strategy.rs)     │
          │  • Ball-path prediction   │
          │  • Attacker / defender    │
          │    role assignment        │
          │  • PID-based control      │
          └─────────────┬─────────────┘
                        │  uses
          ┌─────────────▼─────────────┐
          │  Physics / Math modules   │
          │  Vec2 · Vec3 · AngDeg     │
          │  Circle2 · Seg2 · Line2   │
          │  DAN (Distance & Normal)  │
          │  Simulation               │
          └───────────────────────────┘
```

---

## Project Structure

```
RAIC2019/
├── src/
│   ├── main.rs                 # Entry point & game-loop runner
│   ├── strategy.rs             # Strategy trait definition
│   ├── my_strategy.rs          # Parsian strategy implementation
│   ├── remote_process_client.rs# TCP client & JSON codec
│   ├── simulation.rs           # Physics simulation (collisions, movement)
│   ├── dan.rs                  # Distance-And-Normal arena queries
│   ├── pid.rs                  # PID controller
│   ├── draw.rs                 # Custom rendering helpers
│   ├── entity.rs / entity3.rs  # 2-D / 3-D entity abstractions
│   ├── vec2.rs / vec3.rs       # 2-D / 3-D vector math
│   ├── angdeg.rs               # Angle utilities (degrees)
│   ├── circle2.rs              # 2-D circle geometry
│   ├── seg2.rs / line2.rs      # 2-D segment & line geometry
│   ├── def.rs                  # Default implementations & helpers
│   └── model/                  # Game-model structs (JSON-serialisable)
│       ├── action.rs
│       ├── arena.rs
│       ├── ball.rs
│       ├── game.rs
│       ├── nitro_pack.rs
│       ├── player.rs
│       ├── robot.rs
│       └── rules.rs
├── Cargo.toml                  # Rust package manifest
├── Dockerfile                  # Container image for compilation & execution
├── compile-in-docker.sh        # Build inside the official Docker image
├── run-in-docker.sh            # Run inside Docker (used by the judge)
├── runp1.sh                    # Run player 1 (port 31001, default token)
└── runp2.sh                    # Run player 2 (port 31002, default token)
```

---

## Prerequisites

| Tool | Minimum version | Notes |
|------|-----------------|-------|
| [Rust](https://rustup.rs/) | 1.31 | Edition 2018 |
| [Cargo](https://doc.rust-lang.org/cargo/) | bundled with Rust | – |
| [Docker](https://www.docker.com/) *(optional)* | any recent | for containerised builds |

---

## Building

### Native Build

```bash
cargo build --release
```

The compiled binary is placed at `target/release/my-strategy`.

### Docker Build

```bash
# Build the Docker image (compiles the project inside the container)
docker build -t parsian-raic2019 .

# Or use the helper script (mirrors the judge's build environment)
bash compile-in-docker.sh base
```

---

## Running

### Against the Local Server

```bash
# Syntax: ./my-strategy <host> <port> <token>
./target/release/my-strategy 127.0.0.1 31001 0000000000000000
```

If no arguments are supplied the binary defaults to `127.0.0.1:31001` with the
token `0000000000000000`.

### Using Helper Scripts

```bash
# Player 1 — connects to port 31001
bash runp1.sh

# Player 2 — connects to port 31002
bash runp2.sh
```

---

## Key Modules

### `Strategy` trait (`src/strategy.rs`)

Defines the interface every strategy must implement:

```rust
pub trait Strategy {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action);
    fn custom_rendering(&mut self) -> String { String::new() }
}
```

Implement `act` to compute the desired `Action` for a given robot each tick.
Override `custom_rendering` to stream debug shapes to the visualiser.

### `MyStrategy` (`src/my_strategy.rs`)

The Parsian team's concrete implementation.  Key behaviours:

- **Ball-path prediction** — rolls the physics simulation forward
  `BALL_PREDICTION_TICKS` (150) ticks every other tick.
- **Role assignment** — robots with `id ∈ {1, 3}` play attacker;
  others play defender.
- **PID control** — smooth velocity targeting via `pid.rs`.
- **Custom rendering** — optional debug overlay (enabled by setting
  `CAN_DRAW = true` in `my_strategy.rs`).

### `Simulation` (`src/simulation.rs`)

A lightweight physics engine that mirrors the server's internal model:

| Method | Purpose |
|--------|---------|
| `tick` | Advance one full game tick for a single robot |
| `tick_game` | Advance one full game tick for all entities |
| `get_ball_path` | Return the predicted ball positions for N ticks |
| `collide_entities` | Elastic collision resolution between two entities |
| `collide_with_arena` | Arena wall collision & bounce |
| `move_e` | Euler-integrate position & velocity, apply gravity |

### `DAN` (`src/dan.rs`)

Computes the **distance and outward normal** from any 3-D point to the
nearest surface of the arena.  Used by `Simulation` for all wall-collision
checks.

---

## Contributing

1. Fork the repository and create a feature branch.
2. Implement your strategy by modifying (or replacing) `src/my_strategy.rs`.
3. Run `cargo build --release` and test against a local game server.
4. Open a pull request describing your changes.

---

*Parsian team — RAIC 2019*
