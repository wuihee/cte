# Combat Training Engine

UFC fighter rankings using an enhanced Elo rating system.

## Quick Start

```sh
# Setup
chmod u+x setup.sh && ./setup.sh

# Sync data (first time - takes a while)
cargo run -- --sync

# Run TUI
cargo run
```

## Algorithm

Enhanced Elo with:

- **Dynamic K-factor**: 32 for new fighters, 16 for veterans (15+ fights)
- **Finish bonuses**: KO/TKO 1.25x, SUB 1.20x
- **Early finish**: First round finishes 1.15x
- **Dominance factor**: Up to 1.3x based on stats (knockdowns, strikes, control, takedowns)

## Testing

```sh
cargo test
```
