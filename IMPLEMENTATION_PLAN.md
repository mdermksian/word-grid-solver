# Word Grid Implementation Plan

Last updated: 2026-09-07

This is the living execution plan for the architecture described in
`ARCHITECTURE.md`. It tracks outcomes and dependencies, not a chronological log
of decisions.

## Status and Maintenance

- `[ ]` pending
- `[~]` in progress
- `[x]` complete
- `[!]` blocked; add a short reason on the same line

Update this file in the same change that completes or materially revises a task.
Add newly discovered work under the phase whose acceptance gate it affects. If a
change conflicts with `ARCHITECTURE.md`, update and review the architectural
vision before implementing the conflicting change.

## Preserved Behavior Baseline

The following behavior must continue working throughout Phase 1:

- [x] Launch the desktop Bevy game through `cargo run -p word-grid-game`
- [x] Render the 4x4 Standard New board with correctly labeled cubes.
- [x] Select adjacent cubes without reusing a cube and type words by keyboard.
- [x] Submit with Enter, cancel with Escape, and edit input with Backspace.
- [x] Validate board paths and dictionary membership and reject repeated words.
- [x] Display feedback, temporary accepted-word paths, found words, and score.
- [x] Scroll the found-word list.
- [x] Preserve solver CLI arguments and terminal output.

## Phase 1 — Document and Establish the Foundation

Dependencies: none. Complete these increments in order unless a task explicitly
states otherwise.

### 1. Architecture and planning baseline

- [x] Add `ARCHITECTURE.md` with goals, boundaries, state ownership, data flow,
  platform policy, testing strategy, and future extension seams.
- [x] Add this living plan with observable tasks and phase gates.
- [x] Reconcile both documents with the completed Phase 1 code before closing the
  phase.

### 2. Extract and redesign game core

- [x] Add `word-grid-game-core` as a workspace crate that has no Bevy or
  filesystem dependency.
- [x] Add validated dynamic `CubeSet`, `ScoringTable`, `PlayMode`, and
  `GameRules` types with Standard New, Standard Old, and standard scoring
  built-ins.
- [x] Add authoritative `BoardRoll`/`RolledCube` generation using an injected RNG.
- [x] Add the multiplayer-capable `Match` aggregate, phase transitions, injected
  timer advancement, per-player submissions, review reconciliation, round
  snapshots, and match totals.
- [x] Cover invalid configuration, seeded rolling, phase errors, timing, `Qu`,
  scoring thresholds, duplicates, reconciliation, and multi-round totals in
  focused core tests.

### 3. Clean up solver and migrate CLI

Dependencies: game-core scoring types must exist before the CLI migration.

- [x] Make dictionary parsing available from text/bytes while retaining the
  native file convenience API at the solver edge.
- [x] Return deterministic word-and-path results from discovery and remove score
  ownership from `GridSolver`.
- [x] Move standard score calculation to game core and update the CLI without
  changing its arguments or visible output.
- [x] Update solver unit tests and CLI integration tests for the new interfaces.

### 4. Decompose the Bevy client

Dependencies: game-core match and solver APIs are stable enough for integration.

- [x] Reduce `main.rs` to app launch and expose client composition from the game
  library.
- [x] Add Content, Flow, Match, Board, and HUD plugins with narrow ownership.
- [x] Add screen/round states, ordered Input/Domain/Presentation system sets,
  typed `PlayerIntent`, and typed `MatchNotice` messages.
- [x] Make MatchPlugin the sole mutator of the authoritative `Match` resource.
- [x] Replace fixed grid-position arrays with layouts derived from board size.
- [x] Make the board render the authoritative `BoardRoll`; remove independent
  presentation randomness and grid replacement.
- [x] Preserve selection, typing, feedback, highlight, word-list, scrolling, and
  score behavior through the new boundaries.
- [x] Add headless bridge tests plus pure tests for presentation helpers.

### 5. Consolidate assets and integrate

- [x] Consolidate runtime fonts, models, and dictionary content under root
  `assets/` and load runtime content through the client boundary.
- [x] Preserve native executable-relative asset discovery without introducing
  paths in domain crates.
- [x] Update packaging and project documentation for the resulting asset layout
  and workspace crates.

### 6. Phase 1 verification gate

- [x] Mark every preserved baseline behavior above complete after a smoke test or
  automated equivalent.
- [x] Verify there is one authoritative roll and `game-core` has no Bevy or
  platform-I/O dependency.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [x] Run `cargo test --workspace --locked`.
- [x] Run `cargo check --workspace --all-targets --locked`.
- [ ] Confirm `ARCHITECTURE.md` describes the implementation and close Phase 1.

## Phase 2 — Complete Solo Round

Dependencies: Phase 1 verification gate.

- [ ] Add loading and main-menu screens.
- [ ] Allow selection of the Normal preset before constructing a match.
- [ ] Animate cubes toward the already-authoritative roll.
- [ ] Start authoritative time and timed music when rolling completes.
- [ ] Transition automatically to round review when time expires.
- [ ] Add next-round and finish-match actions with cumulative totals.
- [ ] Add integration tests for a complete multi-round solo match.

Acceptance gate: a player can complete multiple timed rounds and view the final
match total without restarting the application.

## Phase 3 — Modes and Configuration

Dependencies: complete solo round flow.

- [ ] Add Standard Old and Big preset selection and presentation.
- [ ] Add custom square size, cube set, minimum length, duration, scoring table,
  and dictionary selection.
- [ ] Add Endless as a play mode over the existing configuration and submission
  engine.
- [ ] Define versioned external-content DTOs only when file-based customization
  is implemented; validate DTOs into existing core values.
- [ ] Test all `GAME.md` modes without presentation-side rule special cases.

Acceptance gate: Normal, Big, Custom, and Endless games can be constructed and
played through shared domain and presentation paths.

## Phase 4 — Local Multiplayer

Dependencies: round review and configuration flows.

- [ ] Add player setup and stable local input ownership.
- [ ] Add per-player word entry and review views.
- [ ] Present duplicate cancellation and adjusted round scores.
- [ ] Add cumulative standings across rounds.
- [ ] Verify reconciliation and totals against the same game-core fixtures used
  by UI integration tests.

Acceptance gate: a local multiplayer match produces the same duplicate and score
results in the UI and game-core tests.

## Phase 5 — Online and Further Growth

Dependencies: begin only when a concrete product requirement selects authority,
transport, and supported platforms.

- [ ] Add a server/transport adapter around `Match` without network types in game
  core.
- [ ] Define wire commands for player actions and authoritative outcomes/rolls.
- [ ] Add persistence, settings, accessibility, analytics, and additional modes
  as independent adapters or plugins when prioritized.
- [ ] Extend both planning documents before starting work that changes dependency
  direction or authoritative state ownership.

Acceptance gate: each selected capability integrates through an established
adapter boundary without moving platform concerns into the inner crates.
