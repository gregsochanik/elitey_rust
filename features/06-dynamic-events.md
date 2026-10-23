# Feature 06: Dynamic Events While Jumping

## Overview
Surprise and tension between systems. Each jump can trigger an event in two phases: during hyperspace and on approach. The destination's government and economy change how often events happen.

## Status
Implemented in `src/game/events.rs` and `src/game/travel.rs`. Gaps are listed at the end and tracked as issues.

## Requirements

### Two-Phase Events
- [x] Jump phase (hyperspace), base chance 10%
- [x] Approach phase (after the jump, before docking), base chance 12%
- [x] The phases roll independently: a player can get both, either or neither
- [x] Each event offers choices; each choice has an outcome
- [x] Outcomes can change credits, hull, fuel and reputation
- [x] Jump and approach events are shown with their phase
- [x] Outcomes can remove or award cargo
- [ ] Outcomes can move the player to another system (see Gaps)

### Events
Jump: Wormhole, Hyperspace Anomaly, Navigation Error, Space Debris.
Approach: Alien Encounter, Pirate Attack, Rescue Signal, Government Inspection, Military Patrol.

- [x] All nine events exist, with choices
- [x] Jump events only occur in the jump phase, approach events in the approach phase

| Event | Choices |
|---|---|
| Wormhole | Enter, avoid |
| Hyperspace Anomaly | Investigate (+credits, +2 tons Computers), avoid |
| Navigation Error | Correct course (fuel), continue |
| Space Debris | Salvage (+credits, +3 tons Minerals, minor damage), avoid |
| Alien Encounter | Communicate (+credits, +reputation), flee, fight |
| Pirate Attack | Fight, flee (lose up to 3 tons), surrender (lose up to 5 tons and 200cr) |
| Rescue Signal | Investigate (+credits, +reputation, +2 tons Luxury), ignore |
| Government Inspection | Comply (+reputation), bribe, resist (lose all Narcotics, −reputation) |
| Military Patrol | Identify, evade, engage |

### Contextual Modifiers
Frequency multipliers by destination, applied to both phases.

| Government | × | Economy | × |
|---|---|---|---|
| Anarchy | 1.3 | Poor Agricultural | 1.4 |
| Communist | 1.2 | Poor Industrial | 1.2 |
| Dictatorship | 1.1 | Mainly Agricultural / Industrial | 1.1 |
| Corporate State | 0.9 | Rich Agricultural | 0.9 |
| Others | 1.0 | Rich Industrial | 0.8 |
| | | Others | 1.0 |

- [x] Government and economy scale event frequency
- [ ] Government and economy change which events occur (see Gaps)

### Cargo
Losses come from the largest cargo first, or all of one commodity; gains are limited by free space in the hold.

### Reputation
- [x] Outcomes change the commander's reputation, which is saved and shown in `status`
- [ ] Reputation affects outcomes or anything else (Feature 10)

## Gaps
- **Which events occur.** Modifiers only change frequency; every system draws from the same events. The original design had communist systems inspect more and anarchy systems attack more.
- **Movement.** The wormhole and navigation error do not change where the player ends up; the destination is always reached.
- **Advanced:** difficulty scaling, event chains, event history.

## Testing
- [x] Event creation, phase filtering, contextual modifiers and outcome processing have unit tests
- [x] Jump fuel cost is measured from the original system, even after a wormhole
- [ ] Probability and distribution tests for the two phases

## Dependencies
- Feature 02: Star Systems & Travel ✅
- Feature 03: Trading System ✅
- Feature 04: Ship Upgrades ✅
