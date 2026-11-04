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

The desktop client loads its content before showing Play game, Solve board, and
Quit. Play game opens Game Setup, with Standard
(4x4, three-letter minimum), Big (5x5, four-letter minimum), and Custom
options in the left sidebar. Selecting a preset shows its grid dimensions, word
minimum, timing, scoring table, cube-set name, dice count, and every die's faces
in a scrollable panel on the right. Start game in the lower left launches the
selected preset; Back to main menu remains in the top left. Big uses the built-in
25-die Big cube set; Standard uses the Standard New cubes.

Custom starts with Standard rules. Choose a 4x4 or 5x5 grid (which automatically
uses the Standard New or Big cubes), enter a minimum word length of at least
three, and choose a positive whole number of seconds per round or Endless.
Set nonnegative points for the existing 3-4, 5, 6, 7, and 8+ letter buckets.
Click a numeric field to replace its value; Tab/Shift+Tab move between fields,
Ctrl+A selects the whole value, and Enter/Escape leave the field. Invalid values
show an error and disable Start game. Custom edits remain while switching presets
within the setup screen and reset when you reopen it.
Endless rounds show NO LIMIT and continue until you press END ROUND.

Each round animates the authoritative board roll, starts the configured timer
after the dice settle (unless Endless), accepts clicked paths or typed words, and ends in a review
screen when time expires or the player ends the round early. Review keeps the
played board visible beside the complete solution list; clicking a solution
draws its path on the board. You can play additional rounds, finish the match to
see the cumulative score, and return to the menu without restarting the
application.

Solve board opens a separate screen with a 4x4 or 5x5 input grid on the left and
the game's scrollable solution list on the right. Typing a letter into an empty
tile automatically advances to the next tile. Hold either Shift key while typing
to keep focus on the current tile for multi-letter entries such as QU, then use
Tab or Enter to move on. Shift+Tab moves back; arrow keys and clicking let you
revisit tiles. Editing an already-filled tile keeps focus there. Q and QU remain
distinct entries.
Solve grid becomes available once all tiles are filled. Solving runs in the
background using the same dictionary, minimum word lengths (3 for 4x4 and 4 for
5x5), and standard scoring as the presets. Results sort longest first, then
alphabetically. Click a word to highlight and number its path in the grid.
Solve another board clears the grid and results while keeping the selected size;
Back to main menu is available throughout, including during a solve.

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
