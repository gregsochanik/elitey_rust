# Elite Rust – Spec & Roadmap

Terminal Elite-style space trading game: explore systems, trade commodities, survive dynamic events. Feature detail lives in [features/](features/); this file is the plan that orders them. Workflow rules are in [AGENTS.md](AGENTS.md).

## Scope

- Single-player, command-driven CLI (`jump`, `buy`, `sell`, ...).
- Turn-based; no real-time simulation.
- Local save files only.

Out of scope: multiplayer, graphics, real-time combat.

## Status

| # | Feature | Status | Depends on |
|---|---------|--------|------------|
| 01 | Core gameplay | Done | – |
| 02 | Star systems & travel | Done | 01 |
| 03 | Trading | Done | 01, 02 |
| 04 | Ship upgrades | Done | 03 |
| 05 | Save/load | Done | 01 |
| 06 | Dynamic events | Done (reputation effects not applied until 10) | 02 |
| 07 | Weapons & equipment (design) | Spec only | 04 |
| 09 | Equipment system (implementation) | Spec only | 04, 07 |
| 08 | Combat | Spec only | 06, 09 |
| 10 | Reputation | Spec only | 03, 06 |
| 11 | Trading log (v1: record, `log`, persist) | Done (v1) | 03, 05 |
| 13 | UI improvements | Spec only | – |

## Roadmap

1. **Equipment (07 + 09).** 07 is the design; 09 is the implementation to build. Slots, power, shop, install/remove, damage/repair, `equipment` command, save/load of equipment.
2. **Combat (08).** Turn-based encounters triggered from travel events (06), using equipment stats. Escape, loot, and defeat consequences.
3. **Trading log (11).** Transaction history and profit/loss. Independent of 07–09, so it can be done in parallel or first as a low-risk win.
4. **Reputation (10).** Price modifiers, criminal record, event outcomes. Hooks into trading, events and combat, so it comes after them.
5. **UI improvements (13).** Colour, table alignment and status indicators using existing crates (`colored`, `crossterm`). Can land at any point; doing it last avoids reworking output for new commands.

## Open decisions

- **Async runtime.** `main` uses `#[tokio::main]` but the game is synchronous. Keep it, or drop `tokio` until something needs it.
- **Event reputation.** Events set `reputation_change` but nothing applies it. Wire up in 10, or earlier if wanted.
- **Overlap.** 07 and 09 both describe equipment; consider merging into one spec when starting step 1.
- **Numbering.** There is no feature 12. Either reserve it or renumber.
- **Tests.** The README references a `tests/` directory that does not exist. Add BDD-style integration tests there as each feature lands.

## Definition of done (per feature)

1. Spec in `features/` is current.
2. BDD-style tests written first and passing.
3. `cargo test`, `cargo clippy` and `cargo fmt` clean.
4. README status and command list updated.
5. Unused code removed.
