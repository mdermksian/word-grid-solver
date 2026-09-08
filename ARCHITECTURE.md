# Word Grid Architecture

## Purpose

This document defines the long-term architectural boundaries for the Word Grid
workspace. It describes why the boundaries exist and which subsystem owns each
kind of state. Execution status belongs in `IMPLEMENTATION_PLAN.md`.

The architecture optimizes for readable game rules, deterministic tests, and a
Bevy client that can grow without turning its entry point into a collection of
unrelated systems.

## Goals

- Keep game rules usable and testable without Bevy, rendering, audio, a window,
  the filesystem, or wall-clock time.
- Maintain one authoritative representation of a match and board roll.
- Organize client code by cohesive game-facing capabilities.
- Support square boards and cube sets of different sizes without special cases.
- Model multiple players now while delivering a single-player client first.
- Keep platform I/O at the edge so a future web client remains feasible.
- Make later custom content and networking adapters additive rather than reasons
  to rewrite the rules engine.

## Non-goals

- Phase 1 does not add menus, polished rolling, music, custom-content file
  formats, networking, persistence, or accounts.
- The core is not an event-sourced framework and does not introduce traits for
  every operation. Abstractions are added at real nondeterministic or platform
  boundaries.
- The Bevy client is not required to be reusable as a general-purpose engine
  plugin outside this workspace.

## Dependency Direction

Dependencies point toward pure logic. Inner crates never import outer adapters.

```text
word-grid-solver CLI ───────────────┐
                                    v
word-grid-game client ──> word-grid-game-core ──> word-grid-solver-core
         |
         └──> Bevy, rendering, audio, assets, platform APIs
```

### `word-grid-solver-core`

Owns reusable word-grid primitives: dictionaries, normalized square grids, path
validation, neighbor traversal, and deterministic discovery of words and paths.
It does not decide how a game scores or reconciles a discovered word.

### `word-grid-game-core`

Owns the authoritative game model: validated rules, cube sets, scoring tables,
board rolls, players, match and round phases, elapsed game time, submissions,
duplicate cancellation, round results, and match totals. It depends on the
solver's grid and dictionary primitives but has no Bevy or filesystem dependency.

### `word-grid-game`

Owns composition and adaptation: loading content, translating player input into
domain operations, rendering the board, HUD presentation, animation, audio, and
platform-specific asset setup. Bevy resources may wrap core values, but they do
not reimplement the rules.

### `word-grid-solver` CLI

Owns argument parsing and terminal output. It uses the solver for discovery and
the game core's standard scoring table so scoring behavior stays consistent.

## Authoritative State

`Match` is the authority for rules-visible state. It owns the board, match phase,
round timer, accepted submissions, reconciliation results, and scores. Its fields
are private so invalid phase transitions cannot be manufactured by callers.

Bevy owns presentation-only state such as the current text draft, selected-cell
draft, entity handles, scroll position, animation timers, and temporary visual
highlights. Bevy screen states are projections of `MatchPhase`; they exist to
schedule systems and clean up entities, not to decide whether a move is legal.

The board roll follows one direction only:

```text
seeded/platform RNG
       |
       v
Match::start_round -> BoardRoll -> WordGrid
                         |
                         v
              BoardPlugin entities/animation
```

The client never rolls labels independently and never overwrites the domain grid
with a rendered result.

## Domain Model

- `GameRules` combines `PlayMode`, board size, minimum word length, round time,
  `CubeSet`, and `ScoringTable` after validating their cross-field invariants.
- `CubeSet` contains owned cubes with one or more nonempty faces. Built-in sets
  happen to use six faces, but the model does not require that for custom sets.
- `BoardRoll` stores one `RolledCube` per row-major board position. Each entry
  records the source cube, chosen face, and resulting label.
- `Match` supports one or more stable `PlayerId` values and moves through Ready,
  Rolling, Playing, Review, and Complete phases.
- A timed round begins accepting submissions only after the client reports that
  rolling has completed. The core advances using injected elapsed durations.
- Submissions are normalized and validated against the authoritative grid and a
  caller-provided dictionary. Duplicate submissions by one player fail at once.
- At review, words submitted by more than one player are canceled for everyone;
  remaining words are scored using the configured table.
- Completed `RoundResult` values are immutable snapshots used to calculate match
  totals. Presentation derives views from these snapshots.

## Bevy Client Boundaries

The client is composed from plugins with one clear lifecycle each:

- `ContentPlugin` provides built-in content and platform-neutral loading.
- `FlowPlugin` owns top-level `Screen` and match-only `RoundScreen` projections.
- `MatchPlugin` owns `ActiveMatch` and is the only plugin that mutates it.
- `BoardPlugin` owns cube presentation, picking, layouts, highlights, and future
  roll animation.
- `HudPlugin` owns the input draft, feedback, found-word list, and scores.
- `AudioPlugin` will react to match notices when audio is introduced.

Plugins may use private helper modules rather than creating a plugin for every
file. Cross-plugin behavior uses typed messages and an explicit schedule:

```text
GameSet::Input       GameSet::Domain       GameSet::Presentation
device/picking  ->   PlayerIntent     ->   MatchNotice
draft updates        Match mutation        HUD / board / audio
```

`PlayerIntent` expresses a user action without containing Bevy entities.
`MatchNotice` reports domain outcomes that may have several presentation
consumers. Direct domain mutation from board or HUD systems is not permitted.

Top-level screens are Loading, Menu, and Match. While a match screen is active,
its projected round screen is Rolling, Playing, or Review. Phase 1 may enter the
match screen directly to preserve today's runnable shell; later phases add the
loading and menu experiences.

## Content and Platform Boundaries

Runtime assets live under the workspace `assets/` directory and are addressed
through Bevy's asset system. Native executable-relative asset discovery belongs
to the desktop composition edge. Domain crates accept parsed text or values and
do not resolve paths.

Built-in cube sets, presets, and scoring tables are Rust data initially. A future
content adapter may deserialize versioned DTOs, validate them into existing core
types, and report validation errors without changing match logic. DTOs and asset
handles must not become fields in domain types.

A future online server will own the same `Match` aggregate and translate wire
commands into its public operations. Clients exchange player actions and
authoritative outcomes or board rolls. Transport types, sockets, and
serialization policy remain outside game core.

## Testing and Change Rules

- Rule combinations and state transitions are tested in game core using seeded
  randomness and injected elapsed time.
- Grid traversal and deterministic word paths are tested in solver core.
- Bevy's domain bridge is testable in a headless app independently of rendering.
- Rendering helpers remain pure where practical; the runnable client receives a
  manual smoke test when presentation changes.
- CLI integration tests protect terminal compatibility.
- Every phase must pass formatting, Clippy with warnings denied, workspace tests,
  and all-target compilation.

If a feature requires reversing a dependency, duplicating authoritative state,
or moving platform I/O into an inner crate, update this document and resolve the
architectural conflict before implementing the feature. Progress and newly
discovered tasks are maintained in `IMPLEMENTATION_PLAN.md`.

## Intentional Tradeoffs

- The design is pragmatic rather than strictly hexagonal: core methods form the
  application boundary, while typed Bevy messages isolate presentation concerns.
- Concrete domain values are preferred over speculative strategy traits. Traits
  are introduced only when multiple implementations or an external boundary
  justifies them.
- Desktop packaging is supported first, while content APIs avoid assumptions
  that would prevent a later WASM build.
- The core is local-first and online-ready: multiplayer rules are represented
  now, but authority and transport infrastructure are deferred.
