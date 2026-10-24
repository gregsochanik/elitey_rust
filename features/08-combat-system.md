# Feature 08: Combat

## Overview
Turn-based ship combat. Version 1 is a small, complete slice: one pirate fight, driven by the equipment the player has installed. Everything else from the original design (initiative, heat, power, positioning, special actions, other enemies) comes later.

## Status
Version 1 implemented: Pirate Attack's Fight choice starts a real fight. Numbers were tuned by simulation; see Balance.

## Version 1 Scope

### Trigger
- [x] The Pirate Attack event's **Fight** choice starts combat instead of resolving instantly
- [x] **Flee** and **Surrender** on that event stay as they are

### Combatants
Player stats come from the ship and installed equipment (Feature 07):
- **Hull:** the ship's hull percentage (0–100); persists after combat
- **Shield:** points equal to the installed shield's Protection, refilled at the start of every fight
- **Shield recharge:** the installed shield's Recharge, restored at the end of each round, up to the maximum
- **Weapons:** each installed weapon, with its Damage and Accuracy (no shield means no shield points)

| Pirate Raider | |
|---|---|
| Hull | 70 |
| Shield | 25 (recharge 4) |
| Weapon | damage 14, accuracy 0.65 |

Tuned by simulation (see Balance).

### Round
1. **Player turn:** choose attack, flee or surrender.
2. **Enemy turn:** the raider attacks (skipped if it was destroyed or the player left).
3. **Recharge:** both sides' shields recharge.

- [x] **Attack:** every installed weapon fires once. A shot hits with probability equal to its Accuracy. Damage hits shield points first, then hull
- [x] **Flee:** 50% chance to escape. On failure the raider attacks once more as normal. Escape ends combat with no reward or penalty
- [x] **Surrender:** ends combat; lose up to 5 tons of cargo (largest first) and 200 credits, no hull damage
- [x] Attacking with no weapons installed is refused with a message, and takes no turn
- [x] Combat shows both sides' hull and shield each round, and a log of what happened

### Outcomes
- [x] **Victory** (raider hull reaches 0): +200–400 credits, 1–3 tons of a random commodity as loot (limited by free space), +5 reputation
- [x] **Defeat** (player hull reaches 0): lose all cargo and half the credits, hull left at 10%, −5 reputation. The game does not end
- [x] Shields refill after combat; hull does not

### Rules
- [x] The combat rules are pure functions of the state and a random number generator, so tests can fix the dice
- [x] No saving or loading during a fight; a fight is resolved before returning to the game
- [x] Equipment drives the result: an unarmed ship cannot win, and better weapons and shields win more often

## Later
Kept from the original design, not in v1:
- Other enemies: Corporate Security, Alien Scouts, Military Patrol, each with its own tactics and special ability
- Initiative, critical hits, dodge and speed from engines and sensors
- Heat and power management, damage types
- Special actions: precise and heavy attacks, targeting systems, emergency repair, equipment abilities
- Starting combat from other events (Alien Encounter, Military Patrol)
- Difficulty levels, positioning and cover (descoped)

## Balance
Attacking every round against the Pirate Raider, 4,000 fights per loadout (hull lost is out of 100):

| Loadout | Wins | Average hull lost |
|---|---|---|
| Starting (Pulse Laser, Basic Shield) | 100% | 28 |
| No shield | 86% | 79 |
| Standard Shield | 100% | 2 |
| Pulse + Beam Laser | 100% | 3 |
| Beam Laser only | 100% | 13 |

An unarmed ship cannot attack, so it cannot win. The first Raider (hull 60, shield 20, damage 10) was too weak: the starting ship won every fight losing 3 hull, because shield recharge absorbed nearly all its fire. Heavier tuning (damage 16–20) made the starting ship lose 40% or more of fights, which is too punishing while repair doesn't exist.

Tests in `tests/combat_balance.rs` keep these properties.

## Open Questions
- **Ship destroyed.** v1 never ends the game on defeat. Should a later version, and what does permadeath look like with save slots?
- **Repair.** Hull stays damaged until repair exists (Feature 07, later). The starting ship loses about 28 hull per fight, so three or four fights wear it down.

## Acceptance Criteria
- [x] Choosing Fight against pirates plays a fight round by round
- [x] The starting loadout beats a Pirate Raider in most fights, losing some hull
- [x] A ship with no weapons cannot win, but can flee or surrender
- [x] Victory, defeat, flee and surrender each leave credits, cargo, hull and reputation as specified
- [x] Better equipment visibly improves results

## Testing (write first)
- [ ] Attack with fixed dice: hit and miss, shield absorbs before hull, damage carries over when the shield breaks
- [ ] Shield recharge never exceeds the maximum
- [ ] Round order: a destroyed enemy does not attack
- [ ] Flee success and failure
- [ ] Surrender takes the stated cargo and credits
- [ ] Victory and defeat consequences
- [ ] No weapons: attack refused
- [ ] Simulation over many seeds: starting loadout wins at least 80%; unarmed ship wins 0%; an upgraded ship wins more than the starting one
- [ ] Pirate Attack's Fight choice starts combat

## Dependencies
- Feature 06: Dynamic Events ✅ (Pirate Attack)
- Feature 07: Equipment ✅ (weapons, shields)
