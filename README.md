# word-grid-solver

Find and score words on a square letter grid. The repository is a Cargo
workspace with a reusable solver library, a pure game-rules library, a CLI, and
a Bevy game client.

The long-term dependency and state-ownership rules are documented in
[`ARCHITECTURE.md`](ARCHITECTURE.md). Incremental implementation status and the
feature roadmap live in [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md).

## Workspace structure

- `crates/solver`: dictionaries, grids, paths, and deterministic word discovery.
- `crates/game-core`: validated rules and authoritative match state without Bevy
  or platform I/O.
- `crates/game`: the plugin-oriented Bevy client.
- `crates/cli`: the command-line solver and standard game scoring adapter.

## CLI usage

```sh
cargo run -p word-grid-solver -- --size 2 --min-length 3 --dict tests/fixtures/words.txt c a t s
```

The word list defaults to `assets/dictionaries/twl06.txt` when `--dict` is not
provided.

## Game

```sh
cargo run -p word-grid-game
```

The desktop client loads its content before presenting the Normal game preset.
Each round animates the authoritative board roll, starts a three-minute timer
after the dice settle, accepts clicked paths or typed words, and ends in a review
screen. You can play additional rounds, finish the match to see the cumulative
score, and return to the menu without restarting the application.

## Local quality checks

The repository pins its Rust toolchain in `rust-toolchain.toml`; Rustup selects it automatically when you run Cargo commands from the repository root. Before pushing a branch, run:

```sh
# Apply Rust formatting changes.
cargo fmt --all

# Verify formatting, lint all workspace targets, run tests, and compile all targets.
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo check --workspace --all-targets --locked
```

## Packaging locally

Build a release archive on the same operating system you intend to run it on. The bundle must keep the game executable next to its `assets/` directory.

On Linux or macOS, run:

```sh
cargo build --release --locked --package word-grid-game

target_name="$(rustc -vV | sed -n 's/^host: //p')"
bundle="dist/word-grid-game-${target_name}"
mkdir -p "${bundle}/assets"
cp target/release/word-grid-game "${bundle}/"
cp -R assets/. "${bundle}/assets/"
cp LICENSE packaging/README.txt "${bundle}/"
tar -C dist -czf "dist/word-grid-game-${target_name}.tar.gz" "word-grid-game-${target_name}"
```

On Windows PowerShell, run:

```powershell
cargo build --release --locked --package word-grid-game

$targetName = rustc -vV | Select-String '^host: ' | ForEach-Object { $_.ToString().Substring(6) }
$bundle = "dist/word-grid-game-$targetName"
New-Item -ItemType Directory -Force -Path "$bundle/assets" | Out-Null
Copy-Item target/release/word-grid-game.exe $bundle
Copy-Item assets/* "$bundle/assets" -Recurse
Copy-Item LICENSE, packaging/README.txt $bundle
Compress-Archive -Path $bundle -DestinationPath "dist/word-grid-game-$targetName.zip"
```

The resulting archive contains the executable, runtime model assets, license, and release notes. For the four official desktop targets, push a `v*` tag and let the release workflow build on each native runner.


## License

This project is licensed under the GNU General Public License v3.0. See [LICENSE](LICENSE) for details.
