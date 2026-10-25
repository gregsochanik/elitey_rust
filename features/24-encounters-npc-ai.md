# Feature 24: Encounters and NPC Behaviour

## Overview
Enemies and other ships decide what to do instead of always shooting. A pirate who is losing can run or beg for mercy; police act on who you are; encounter tables decide who shows up where. All of it is a pure, deterministic Rust function over game state with injected randomness, so it behaves the same from the CLI and the Godot GUI. The GUI only dramatises the outcome.

## Status
Phases 1 to 3 are built (decision function, retreat and surrender, police and traders, encounter tables, bounty hunters, groups, combat-rating scaling). Route planning (#231) and flight AI are open. Umbrella #224.

The brief for this spec asked for `features/16-encounters-npc-ai.md`; 16 is Jobs, so this is Feature 24.

## Current state (checked against the code)
- **The core refactor from Feature 15 is done.** `elite-core` has no terminal I/O, travel is a step machine (`plan_jump`, `start_jump`, `advance_journey`; `Step::Event`, `Step::Fight`, `Step::Inspection`, `Step::Arrived`), and randomness is injected. `fight.rs` is the CLI's terminal driver (`elite-cli/src/cli/fight.rs`).
- **Combat** (`combat.rs`): `Combat { player, enemy, log }`, `Action { Attack, Flee, Surrender }` (the player's), `Outcome { Ongoing, Victory, Defeat, Escaped, Surrendered }`. `resolve_round(action, roll)` resolves the player's action, then the enemy **always fires** with every weapon, then both shields recharge. The enemy has no decision step: it never flees, surrenders or holds fire. Flee succeeds on a 0.5 roll.
- **Enemies are hard-coded:** `Combatant::pirate_raider()` (hull 70, shield 25, recharge 4, one weapon 14 damage at 0.65) and `patrol_vessel()` (identical stats). There is no scaling by system or by player.
- **Encounters** come from travel events (`events.rs`). The Approach phase triggers 12% of the time (Jump 10%), scaled by government and economy (`calculate_contextual_modifier`). The event is then picked by weight: `government_weight` (e.g. Anarchy 2.5x pirates; Communist 2.5x inspections), `economy_weight` (poor systems 1.4x pirates) and `standing_weight` (Criminal standing doubles patrols in lawful systems, halves them in lawless ones). `PirateAttack` has a choice with `starts_combat: true`; other combat starts are a patrol fight after a failed smuggling inspection and bounty-job fights (all against `pirate_raider`).
- **Aftermath** (`aftermath.rs`): `settle` and `settle_with_reward` apply Victory (random credits 100 to 200, 1 to 3 tons of loot, +5 reputation, a recorded kill and possible promotion), Defeat (hold stripped, half credits lost, hull set to 10%, -5 reputation) and Surrendered (5 tons of cargo and up to 200 credits).
- **Tests:** `elite-core/src/game/combat/tests.rs`, `events/tests.rs`, `journey/tests.rs`, `elite-cli/tests/combat_flow.rs` and `combat_balance.rs`. Combat numbers were tuned by simulation (Feature 08, Balance).

## Principles
- NPC decisions are a pure function of game state plus an injected roll. No printing, no stdin, no time, no globals.
- The core returns what the NPC did as data; the CLI prints it and the GUI animates it. Neither makes decisions.
- Behaviour and encounter weights are **data**, so tuning needs no code change.
- Real-time flight AI, if ever built, is a separate view-layer feature that follows the core's decisions.

## Design

### Decision function
```rust
pub fn decide(profile: &Behaviour, npc: &Combatant, player: &Combatant, roll: f64) -> NpcAction
```
`NpcAction` is `Attack`, `Flee` or `Surrender` in v1. Inputs: the NPC's and player's hull and shield percentages, weapon strength, the NPC's behaviour profile and one roll. Later phases add context (cargo value, reputation, legal status) through an `EncounterContext` argument.

Rules, in order:
1. **Surrender** when the NPC's hull is at or below `surrender_below_hull`, or it has no weapons, and `roll < surrender_chance`.
2. **Flee** when hull is at or below `flee_below_hull`, or `odds < flee_odds`, and `roll < flee_chance`. Odds are `npc_power / player_power`, where power is (hull + shield) x damage per round at average accuracy.
3. Otherwise **Attack**.

A decision that no threshold touches consumes **no roll**, so existing seeded fights keep their roll order. `aggression` (0 to 1) shifts `flee_below_hull` and `flee_odds` downward, so aggressive types stand and fight.

### Behaviour as data
`elite-core/data/enemies.json`, read like `ships.json` and `commodities.json`:
```json
{
  "id": "pirate-raider",
  "name": "Pirate Raider",
  "hull": 70, "shield": 25, "recharge": 4,
  "weapons": [{ "damage": 14, "accuracy": 0.65 }],
  "behaviour": {
    "aggression": 1.0,
    "flee_below_hull": 0.0, "flee_odds": 0.0, "flee_chance": 0.5,
    "surrender_below_hull": 0.0, "surrender_chance": 0.0
  }
}
```
Phase 1 loads today's raider and patrol vessel from this file with thresholds set to "never flee or surrender", so behaviour is byte-for-byte what it is now.

### Combat changes
- `Combat::new(player, enemy, behaviour)`. `resolve_round` resolves the player's action; then, if the enemy is still in the fight, calls `decide` and applies the result: Attack fires as today; Flee rolls the enemy's escape; Surrender ends the fight.
- New outcomes `EnemyFled` and `EnemySurrendered`. The player's `Action` and the existing outcomes are unchanged.
- Each round reports what the NPC did (`Combat.last_npc_action`, plus log lines), so the CLI prints "The Pirate Raider turns to run!" and the GUI plays the matching animation.

### Outcomes
Proposed, to be tuned by simulation:

| NPC action | Result |
|---|---|
| Flee succeeds | Fight ends, no loot, no kill; a bounty on that ship is not completed |
| Surrender | Fight ends; reduced credit reward and full cargo loot; counts as a kill for combat rating; a bounty job is completed |
| Attack | Unchanged |

`settle` and `settle_with_reward` gain the two new outcome arms; nothing is duplicated. Police outcomes use the same arms plus a fine (see open questions).

### Encounter generation
- Encounter tables in `elite-core/data/encounters.json`: for each event type and government and economy, a weight, plus the enemy type, count and difficulty. The existing `government_weight`, `economy_weight` and `standing_weight` functions move into this data, with the same numbers.
- **Reputation** (Feature 10) keeps its current role in who shows up (`standing_weight`) and adds a say in behaviour: pirates flee earlier from a high combat rating (Feature 20), police open fire on Criminal standing in lawful systems and wave through Respected commanders.
- Count above 1 is played as consecutive one-on-one fights; multi-target combat is out of scope.
- Everything is deterministic under injected RNG.

### Core API
- `Step::Fight(Combat)` and `Input::FightEnded(Combat)` are unchanged, so the journey and both front ends keep working. New outcomes flow through `FightEnded`.
- `elite-ffi`'s `FightData` gains `enemy_action` (a string such as "attack", "flee", "surrender") and the new outcome names; `elite-godot` passes them to GDScript. The CLI prints the log lines.
- No printing or stdin enters the core.

### Galaxy routing (optional, separate)
`plan_route(from, to, fuel_capacity) -> Option<Vec<Hop>>` in the Rust core: Dijkstra over systems with an edge where the jump cost is within fuel range, minimising total fuel (refuel stops added when a hop exceeds current fuel). Every system has at least two neighbours in range and the galaxy is connected (Feature 22), so a route exists. Shown as a path on the map; each hop is still a separate journey with its own events.

### View layer (later, out of scope for v1)
For a real-time flight view (#187, Feature 23), NPC ships are animated toward the decided action using a small GDScript state machine plus steering behaviours (seek, flee, pursue, evade, arrive, orbit). Revisit when a decision is made to add real-time combat or playable flight with enemies. Godot has no built-in enemy AI; `NavigationServer3D` is for navmesh pathfinding and not a fit for open space (not verified in detail). **LimboAI** (behaviour trees and state machines, a GDExtension with GDScript tasks, MIT, Godot 4.x) is the add-on to reach for only if that state machine grows unwieldy. Beehave and Godot State Charts also exist; I did not verify them.

## Phases

### Phase 1: Decision function and behaviour data
- [ ] `enemies.json` with the raider and patrol vessel; `Combatant` built from it
- [ ] `decide` and `Behaviour`, with unit tests for every threshold
- [ ] `Combat::new` takes a behaviour; `resolve_round` calls `decide`; existing combat tests pass unchanged
- [ ] Today's thresholds ("never flee or surrender") reproduce current seeded fights exactly

### Phase 2: Flee, Surrender and more types
- [ ] `EnemyFled` and `EnemySurrendered` outcomes with settle rules
- [ ] Pirate profile with real thresholds; combat balance re-simulated
- [ ] Trader and police types; police fines
- [ ] CLI and GUI show what the NPC did

### Phase 3: Encounter tables and reputation hooks
- [ ] `encounters.json` replaces the weight functions with the same numbers
- [ ] Count, difficulty scaling, consecutive fights
- [ ] Reputation and combat rating change behaviour as above

### Phase 4 (optional): Galaxy route planning
- [ ] `plan_route` with tests on the real galaxy; map shows the route

### Phase 5 (later, separate): Real-time flight AI in the view layer

## Testing
- Rust unit tests for every decision threshold (each rule, its boundary and the roll cutoff).
- Seeded-RNG tests for encounter table distributions (counts over many rolls within tolerance of the weights).
- Feature-level BDD tests in `elite-cli/tests/` and `elite-core/tests/`, written before implementation. Existing `combat_flow`, `combat_balance`, `economy_balance` and `journey` tests keep passing.

## Acceptance Criteria
- [ ] Changing a threshold in `enemies.json` changes behaviour with no code change
- [ ] Phase 1 leaves every existing test and seeded fight unchanged
- [ ] The same seed gives the same encounter and the same NPC actions
- [ ] The CLI and the Godot GUI show NPC actions from the same core data
- [ ] `elite-core` stays free of terminal I/O (the layering test passes)

## Out of scope
- Real-time combat in this iteration, ML or LLM-driven NPCs, multiplayer
- Multi-target combat
- Balance changes beyond what this spec states

## Decisions
- **Types in v1**: pirate, police and trader. A trader never attacks and flees at the first hit; how the player meets traders (help, trade, rob) is settled when Phase 2 starts, since there is no player-initiated combat yet
- **Bribes**: NPC surrender replaces bribing for v1; inspection bribes (Feature 18) are unchanged
- **Police and bounty hunters**: profiles and fights live here; triggers by standing sit in the encounter table
- **NPC actions**: a separate `NpcAction` enum, sharing the round engine with the player's `Action`

## Dependencies
- Features 06, 07, 08, 10 and the completed core refactor in Feature 15
- Features 18 (inspections), 20 (combat rating), 22 (connected galaxy for routing), 23 and #187 (view layer)
