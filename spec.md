# Elite Rust – Spec & Roadmap

Terminal Elite-style space trading game: explore systems, trade commodities, survive dynamic events. Feature detail lives in [features/](features/); this file is the plan that orders them. Workflow rules are in [AGENTS.md](AGENTS.md).

## Scope

- Single-player, command-driven CLI (`jump`, `buy`, `sell`, ...).
- Turn-based; no real-time simulation.
- Local save files only.

Out of scope: multiplayer, real-time combat. A graphical front end (Godot, Feature 15) is built; ship visuals are Feature 23.

## Status

| # | Feature | Status | Depends on |
|---|---------|--------|------------|
| 01 | Core gameplay | Done | – |
| 02 | Star systems & travel | Done | 01 |
| 03 | Trading | Done | 01, 02 |
| 04 | Ship upgrades | Done | 03 |
| 05 | Save/load | Done | 01 |
| 06 | Dynamic events | Done (gaps in spec) | 02 |
| 07 | Equipment (v1: slots, power, buy/sell, cargo racks) | Done (v1) | 04 |
| 08 | Combat (v1: one pirate fight using equipment) | Done (v1) | 06, 07 |
| 10 | Reputation (v1: standing, illegal-trade penalty, price modifier) | Done (v1) | 03, 06, 08 |
| 11 | Trading log (v1: record, `log`, persist) | Done (v1) | 03, 05 |
| 12 | Ship repair (v1: `repair` for credits) | Done (v1) | 08 |
| 14 | Refuelling (v1: `refuel` for credits) | Done (v1) | 02 |
| 15 | GUI (core refactor, then macOS front end) | Done (Godot front end; wireframe views continue in 23) | 01–08, 10, 11, 12, 14 |
| 13 | UI improvements | Spec only | – |
| 16 | Jobs (v1: board, courier jobs, deadlines) | Done (v1) | 02, 05, 10, 14 |
| 17 | Bounty jobs (v1: pirate hunt jobs using combat) | Done (v1) | 08, 12, 16 |
| 18 | Contract tiers and criminal contracts (v1: freight, smuggling, standing-gated boards) | Done (v1) | 06, 08, 10, 14, 16, 17 |
| 19 | Market dynamics (v1: market memory, price impact, recovery) | Done (v1) | 03, 05, 10, 16 |
| 20 | Combat rating (v1: kills and the Harmless to ELITE rating, shown in `status`) | Done (v1) | 08, 10 |
| 21 | Locked jobs (v1: the board shows jobs the commander cannot take yet, with the reason) | Done (v1) | 16, 18, 20 |
| 22 | Real star systems (v1: 256 real stars from HYG, star classes) | Done (v1) | 02, 05 |
| 23 | GUI ship visuals (ship models as data; wireframe, hidden-line, flat, textured styles) | Spec only | 15, 22 |
| 28 | Navigable galaxy map (pan, zoom, keyboard) | In progress | 15, 22 |
| 27 | Frontier discoveries (v1: frontier stars, finds on first arrival, a discoveries log) | Done (v1) | 22, 25 |
| 26 | Restricted goods and tech-gated equipment (v1: market restrictions, shop and shipyard tech levels) | Done (v1) | 03, 07, 04, 21 |
| 25 | Exploration (v1: visited-stars log, completion percent and rating) | Done (v1) | 02, 05, 22 |
| 24 | Encounters and NPC behaviour (decision function, behaviour data, encounter tables) | Done (v1) | 06, 07, 08, 10, 15, 18, 20 |

## Roadmap

1. **Equipment (07).** v1: slots, power, shop, buy/sell, `equipment` commands, cargo racks, save/load. Effects and damage/repair come later.
2. **Combat (08).** Turn-based encounters triggered from travel events (06), using equipment stats. Escape, loot, and defeat consequences.
3. **Trading log (11).** Transaction history and profit/loss. Independent of 07, so it can be done in parallel or first as a low-risk win.
4. **Reputation (10).** Standing, illegal-trade penalty and price modifiers (v1); criminal record and event outcomes later. Hooks into trading, events and combat, so it comes after them.
5. **UI improvements (13).** Colour, table alignment and status indicators using existing crates (`colored`, `crossterm`). Can land at any point; doing it last avoids reworking output for new commands.
6. **Encounters and NPC behaviour (24).** Enemies decide to attack, flee or surrender from data-driven profiles; encounter tables by government, economy and standing; optional route planning. Combat balance is re-simulated when thresholds turn on.

## Open decisions

- **Numbering.** There is no feature 09 (it was merged into 07). Either reserve the number or renumber.

## Definition of done (per feature)

1. Spec in `features/` is current.
2. BDD-style tests written first and passing.
3. `cargo test`, `cargo clippy` and `cargo fmt` clean.
4. README status and command list updated.
5. Unused code removed.
