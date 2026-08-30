# word-grid-solver

Find and score words on a square letter grid. The repository is now a Cargo workspace with a reusable solver library, a CLI, and a Bevy game shell.

## CLI usage

```sh
cargo run -p word-grid-solver -- --size 2 --min-length 3 --dict tests/fixtures/words.txt c a t s
```

The word list defaults to `twl06.txt` when `--dict` is not provided.

## Game shell

```sh
cargo run -p word-grid-game
```

The game currently starts a minimal Bevy app. Gameplay and solver integration have not been implemented yet.
