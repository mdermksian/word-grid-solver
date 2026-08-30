# word-grid-solver

Find and score words on a square letter grid.

## Usage

```sh
cargo run -- --size 2 --min-length 3 --dict tests/fixtures/words.txt c a t s
```

The word list defaults to `twl06.txt` when `--dict` is not provided.

## Legacy C++ Version

The previous C++ implementation is preserved at `archive/cpp/main.cpp`.
