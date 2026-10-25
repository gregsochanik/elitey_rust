# Feature 15: GUI

## Overview
Add a graphical front end (macOS first, iOS later) alongside the CLI. The game rules must not know which front end is driving them. Today they do: `src/game/*` prints to stdout and reads stdin mid-logic, so a GUI cannot reuse it. This feature is mostly a refactor of the engine into a front-end-agnostic core, followed by a thin GUI on top.

The CLI stays supported and must behave the same from the player's point of view.

## Status
Spec only. Nothing is built.

## Problem (current state)
- 118 `println!` / `print!` calls (plus `colored`) live inside `src/game/*` and `src/data/*` (`trading.rs`, `travel.rs`, `upgrades.rs`, `equipment.rs`, `save_load.rs`, `game_state.rs`).
- Handlers such as `handle_buy`, `handle_sell` and `handle_jump_command` parse `&[String]` args, print their own results and return a `bool`. A caller cannot tell *what* happened.
- Blocking `stdin` reads inside game logic: `travel.rs` (jump confirmation, event choices, combat entry), `save_load.rs` (overwrite/delete confirmation), `game_state.rs` (commander name, load-or-new prompt).
- `main.rs` re-declares `mod data; mod game; mod utils;` instead of using the `lib.rs` exports.
- Good news: `combat.rs` is already a pure state machine (`Combat`, `Action`, `Outcome`, injected `roll`); `fight.rs` is only the I/O driver. `data/*` and helpers like `reachable_destinations`, `calculate_travel_cost` and `fuel_for_distance` are already clean. Use these as the model for the rest.

## Target Architecture
Three crates. `elite-core` depends on no front end; `elite-cli` and `elite-gui` each depend directly on `elite-core`, never on each other:

1. **Core (`elite-core`)**: rules, data and state. No printing, no stdin, no `colored`, no `crossterm`. Typed inputs, typed results, typed errors. Randomness is injected (as `combat` already does) so tests and replays are deterministic.
2. **CLI (`elite-cli`)**: parses command lines, calls the core, formats results as text. All `colored` / `crossterm` code lives here, including what is now `utils/display.rs` and `utils/input.rs`.
3. **GUI (`elite-gui`)**: renders core state and turns clicks into the same typed calls.

### Core API shape
Prefer one small command/outcome surface over many `handle_*` methods:

- Actions are plain data, e.g. `Buy { commodity, quantity }`, `Sell { .. }`, `Refuel { units: Option<u32> }`, `Repair { .. }`, `BuyEquipment { .. }`, `Jump { destination }`.
- Each action returns `Result<Outcome, ActionError>` where the outcome carries what changed (receipt with unit price, total, credits after; fuel used; etc.) and `ActionError` is a `thiserror` enum (`NotEnoughCredits { needed, have }`, `NoCargoSpace`, `UnknownCommodity(String)`, ...). The CLI turns these into today's messages.
- Read-only queries return view structs: `MarketView`, `CargoView`, `StatusView`, `SystemInfoView`, `Vec<Destination>`, `SavesView`, `TradeLogView`. No strings pre-formatted for a terminal.
- **Travel is a step machine, not a blocking call.** `Jump` returns `JumpOutcome::Arrived` or `JumpOutcome::EventPending(PendingEvent)` where the event lists its choices; the caller then submits `ResolveEvent(choice)`, which may return `CombatStarted(Combat)` or a further outcome. The confirmation prompt becomes the caller's job (CLI asks y/n; GUI shows a dialog) before it submits `Jump`.
- **Combat** keeps the existing `Combat` / `Action` / `Outcome` API. `fight.rs` moves to the CLI crate as the terminal driver. When a fight ends, the caller submits the finished `Combat` and its `Outcome` back to the core (`SettleCombat`), which applies today's `settle` rules (credits, loot, reputation, the player's final hull) and returns the session to travel. The full-session test covers this hand-off.
- **Save/load** takes explicit arguments and returns results (`save(slot, overwrite: bool)`, `load(slot)`, `list_saves()`, `delete(slot)`). Confirmation prompts move to the front ends. The save directory comes from a path passed in (or a trait), not hard-coded `dirs` calls scattered through logic, because iOS needs a different location.
- New game takes a commander name as a parameter. The load-or-new prompt moves to the front ends.

## Phases

### Phase 1: Core refactor (in place, single crate)
Done. Typed actions, outcomes, errors and views live in `src/game`; all text, prompts and the terminal fight driver live in `src/cli`. Travel is a step machine (`plan_jump`, `start_jump`, `advance_journey`: events, fights and job arrival are `Step`s answered with `Input`s). Market rolls derive from a world seed, and `tests/core_layering.rs` fails if `src/game` or `src/data` use terminal I/O.
- [x] Introduce the action/outcome/error/view types above and migrate one area at a time: trading, refuel, repair, equipment, upgrades/ships, save/load, trade log, travel and events, combat entry.
- [x] Keep each existing command working through a thin CLI layer that calls the new API and prints. The CLI's visible output may only change where the old text was wrong.
- [x] Remove `println!` / `print!` / `colored` / stdin use from everything under `src/game/` and `src/data/`.
- [x] Make `main.rs` use the library (`use elitey_rust::...`), not duplicate `mod` declarations.
- [x] Inject randomness (travel events, market fluctuation, combat rolls) so core tests can be deterministic.
- [x] Existing `tests/*` still pass. Rule and state tests may move to the core API, but keep representative CLI tests (command parsing, output formatting, prompts, error messages) so the "CLI unchanged" requirement is actually checked.

### Phase 2: Workspace split (done)
- [x] Convert to a Cargo workspace: `elite-core`, `elite-cli` (binary keeps the `elitey_rust` name), later `elite-gui`.
- [x] Update `Makefile`, CI (`.github/workflows/ci.yml`, `release.yml`) and README for the new layout.
- [x] `elite-core` must not depend on `colored`, `crossterm` or `clap`, including as dev or build dependencies. CI fails if `cargo tree -p elite-core -e normal,build,dev` mentions any of them.

### Phase 3: GUI shell (macOS)
- [x] Toolkit: **Godot with gdext** (see Decision). `elite-ffi` is the small facade over the core; `elite-godot` exposes it to GDScript.
- [ ] Screens, all over the core API: Status, Market (buy/sell), Cargo, Galaxy map (reachable systems, fuel cost, jump), Equipment, Ship upgrades/Shipyard, Refuel/Repair, Trade log, Save/Load.
- [ ] Travel events and combat as modal flows driven by `PendingEvent` and `Combat`.
- [ ] Autosave on quit, as in the CLI.

### Phase 4: Wireframe views (later, cosmetic)
- [ ] Galaxy map and system view in the classic wireframe look; flight/docking view. Purely presentation; no new rules.

### Phase 5: iOS (later)
- [ ] Touch layout, save directory via the platform data dir, Xcode build and signing notes.

## Not in scope
- New gameplay, real-time combat, multiplayer, sound.
- Changing game balance. The refactor must not alter rules or numbers.
- Replacing the CLI.

## Open Questions
- **GUI toolkit.** Decided: Godot with gdext; see Decision.
- **When to start.** Phase 1 touches almost every file under `src/game/`, so it conflicts with any feature in flight. Land the remaining reputation issues (feature 10) first, then do Phase 1 as a series of small PRs, one area each.
- **Reputation and UI-13 ordering.** Feature 13 (CLI presentation) should follow Phase 1, so its colour and table work lands in the CLI layer.

## Decision
**Godot 4 with gdext.** Both leading options were spiked with the same Market screen over a small facade (`elite-ffi`: a `Game` object with `market`, `buy`, `sell`, plain records and one error type).

- **SwiftUI over UniFFI** built the Market screen quickly with native tables, but the journey scene between the jump and the station, where events, inspections and fights happen, would need a hand-written Metal or SceneKit renderer, and it is Apple-only. It needs the full Xcode toolchain, not just the command line tools.
- **Godot over gdext** built the same screen with a `Tree` control and added the journey scene (a wireframe ship and a starfield that streaks during a jump) in about 110 lines of GDScript. `elite-godot` wraps `elite_ffi::Game` in about 90 lines, with no binding generator. It also runs on Windows and Linux.

The journey scene is part of every jump, so the engine's 3D, camera, effects and (later) audio count for more than native menus.

**Known risks.** iOS export with gdext is unverified. `godot` crate 0.4.5 is used because 0.5.x needs Rust 1.94; it targets the Godot 4.5 API and runs on 4.7.2. The theme and tables are built by hand rather than native.

**Run the spike.** Install Godot 4, run `cargo build -p elite-godot`, then open `elite-godot/project` in Godot (macOS only for now).

Set `ELITE_SEED=<number>` to make jobs, events and fights repeatable. With `ELITE_SEED=9`, a new game that goes straight to a jump to DISO meets a pirate attack; choosing the first option starts a fight.

## Toolkit options (considered)
The game is mostly menus, tables and dialogs, with a wireframe galaxy map later. Targets are macOS now and iOS later. Check each option's current iOS status before choosing; these tools move quickly.

| Option | Language | Fit | Concerns |
|---|---|---|---|
| **SwiftUI + core via UniFFI** | Swift UI, Rust core | Best native feel and the most reliable macOS and iOS path; menus, tables and dialogs are what SwiftUI is good at; wireframes via Canvas or Metal | Second language; Apple-only; a binding layer to maintain |
| **Godot + godot-rust (gdext)** | GDScript or Rust | Proper scene and UI tooling, 3D wireframes are easy, macOS and iOS export built in; the core loads as a native extension | A second toolchain and editor; extension plus iOS export takes care; less "just Rust" |
| **Bevy** | Rust | Everything in one language and one repo; ECS suits a game; macOS works and iOS is possible | Pre-1.0 with breaking changes each release; large compile times; UI for menu-heavy screens is the weakest part (usually `bevy_egui`) |
| **macroquad + egui** | Rust | Small, quick to start, simple menus and line drawing | Thin iOS support and a smaller ecosystem |
| **eframe (egui)** | Rust | Simplest Rust-only path on macOS | Weak on iOS |
| **Tauri v2 (web UI)** | Rust + HTML/JS | Core stays Rust, UI in web tech; macOS and iOS targets | Web UI stack; wireframe via canvas or WebGL |
| **Slint** | Rust | Declarative native UI, good for menu screens | Mobile support is less mature |

**Leaning:** SwiftUI over the core if the game stays Apple-only; Godot with gdext if a more game-like feel, 3D wireframes or other platforms matter; Bevy if staying all-Rust matters most and the churn is acceptable. Before committing, spike one screen (Market) in the top two once Phase 1 gives a usable core API.

## Acceptance Criteria
- [ ] No `println!`, `print!`, `eprintln!`, `stdin`, `colored` or `crossterm` references under `elite-core` / `src/game` / `src/data`
- [ ] Every action in the CLI command list has a core equivalent returning a typed `Result`, with typed errors for each failure case
- [ ] Travel with an event can be driven entirely from a test via `Jump` then `ResolveEvent(choice)` with injected randomness and no stdin
- [ ] A full session (new game, trade, refuel, jump, fight, save, load) runs in a test through the core API alone
- [ ] CLI behaviour is unchanged for existing flows; `tests/*` green
- [ ] `cargo test`, `cargo clippy` and `cargo fmt` clean at the end of every phase
- [ ] A committed save file written by the pre-refactor game loads in the core (add the fixture and its test before changing the save schema)
- [ ] Phase 3: every command in the README command list is reachable from the GUI, and a save made in the GUI loads in the CLI and vice versa

## Implementation Notes
- Migrate incrementally: one area per commit, tests green each time. Do not do a big-bang rewrite.
- Start with trading (small, well tested in `tests/trading_flow.rs`) to settle the action/outcome/error conventions, then apply them to the rest.
- Travel and events are the hardest part (blocking prompts nested inside `process_travel_with_events`). Do them after the conventions are settled and write the step-machine test first.
- Keep save-file format compatibility: existing saves must still load.
- Follow AGENTS.md: spec first (this file), tests first, remove unused code, update README and `spec.md` when done.

## Dependencies
- Features 01-08, 11, 12, 14 ✅ (all existing commands must be covered)
- Feature 10: Reputation (v1 in progress); its price and standing logic should be in place before the refactor so it is migrated once
- Feature 13: UI Improvements, which concerns CLI presentation. Land it after Phase 1 so its colour/table work is built on the CLI layer, not the engine.
