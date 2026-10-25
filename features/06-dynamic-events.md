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
- [x] Government and economy change which events occur

### Which Events Occur
Each event has a weight at the destination (1.0 is normal; unlisted events stay 1.0). The government and economy weights multiply, and an event is picked in proportion to its weight within the phase.

| Destination | Event | Weight |
|---|---|---|
| Anarchy | Pirate Attack / Inspection / Patrol | 2.5 / 0.2 / 0.3 |
| Communist | Inspection / Patrol / Pirate Attack | 2.5 / 1.5 / 0.6 |
| Dictatorship | Patrol / Inspection / Pirate Attack | 2.5 / 1.5 / 0.7 |
| Corporate State | Inspection / Pirate Attack | 1.5 / 0.5 |
| Poor Agricultural, Poor Industrial | Pirate Attack / Rescue Signal | 1.5 / 1.5 |
| Rich Agricultural, Rich Industrial | Pirate Attack | 0.6 |
| Mainly Industrial | Space Debris | 1.5 |
| Mainly Agricultural | Rescue Signal | 1.3 |

Over 400 jumps each: an anarchy destination had 40 pirate attacks against 10 for a democracy and 6 for a dictatorship, while the dictatorship had more inspections and patrols (15 and 14 against 9 and 8).

### Cargo
Losses come from the largest cargo first, or all of one commodity; gains are limited by free space in the hold.

### Outcome Ranges
The numbers on an outcome are bases; each time an outcome is applied they are rolled.

- **Credits** (gains and fines): base × a roll from 0.6 to 1.4, rounded
- **Cargo tons** (gained or lost): base ± 1 ton, at least 1
- **Windfalls** (a gain with a base of 200 credits or more) are capped at 100 plus half the commander's credits, so a poor commander cannot jump-start the game with one event
- **Reputation, hull and fuel** stay fixed
- The message shows the amounts that were rolled
- Base credits: anomaly 200, debris 100, alien communication 150, alien fight 200, rescue 200, patrol engage 300; fines 50, 100 and 200
- Balance: a player who always takes the best credit choice earns about 18, 26 and 26 credits per jump from events at 100, 500 and 2,000 credits, against about 84 and 151 per jump from trading at 500 and 2,000
- [x] Event income stays a modest share of trading income (simulation)
- [x] Credits and cargo tons fall within their ranges, and windfalls respect the cap
- [x] The message matches the amounts applied

### Reputation
- [x] Outcomes change the commander's reputation, which is saved and shown in `status`
- [ ] Reputation affects outcomes or anything else (Feature 10: prices, and the odds of patrols and inspections, via Feature 18; outcomes later)

## Gaps
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
