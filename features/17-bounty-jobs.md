# Feature 17: Bounty Jobs

## Overview
A second job type for the board (Feature 16): hunt a pirate raider reported at another system. It is the first job that uses combat, so it pays more than a courier job and can cost the player their ship's health, their cargo and their credits.

## Status
In progress. Slices are tracked as GitHub issues; job kinds and rewards are in, bounties reach boards with the fight.

## Version 1 Scope

### On the board
- Each job on a board is a courier job (Feature 16) or, about 30% of the time, a bounty job
- A bounty job is listed as `Bounty: pirate raider near <system>` with the system, distance, fuel, reward and deadline in jumps
- Destinations, deadlines and numbering work exactly as for courier jobs; `accept`, `missions` and `abandon` handle both kinds (`missions` labels the kind)
- Saves written before bounties exist load their jobs as courier jobs

### Reward
- `100 + 12 × fuel cost of the direct jump`, varied by ±10%, rounded to whole credits (about 136cr for a 3-fuel jump)
- Paid on victory **instead of** the random 200–400cr a pirate victory normally pays; the player still keeps the loot and the +5 reputation from combat

### The fight
- When the player arrives at the bounty's system with the job still in time, a fight with a pirate raider starts straight away, after any travel events on that jump
- The fight is the existing combat (Feature 08): same raider, same actions, same equipment effects
- If the player arrives after the deadline, the job has already failed (as for courier jobs) and no fight is forced

### Outcomes
A failed attempt to flee is not an outcome: as in Feature 08 the raider gets a free attack and the fight continues. Only a successful escape ends the fight, and it fails the job with no retry.

| Outcome | Job | Everything else |
|---|---|---|
| Victory | Paid, removed | Loot and +5 reputation as in combat |
| Successful escape | Fails: −3 reputation | Nothing else lost |
| Surrender | Fails: −3 reputation | Cargo and credit loss as in combat |
| Defeat | Fails: −3 reputation | Defeat rules as in combat (cargo lost, half the credits, hull at 10%, −5 reputation) |

- Several bounties for the same system are fought one after another; if the player escapes or surrenders from one, the rest fail with it
- Courier jobs for the same destination are delivered as usual before the fight begins

## Not in v1
- Different enemy strength by reward or system
- Government flavour (lawful systems posting more bounties, anarchies fewer)
- Named targets, targets that move, or bounties that start from events
- Haul and supply jobs, which are tracked separately

## Balance
A bounty should pay clearly more than a courier job (about 55 net per jump at best) because it risks hull, cargo and credits. A fight costs about 28 hull to repair at 2cr per point, so about 56cr, plus fuel. With a base of 100 and 12 per fuel, a 3-fuel bounty nets about 70cr and a 7-fuel one about 140cr after repair and fuel, with a loss rate to be measured.

**Separate issue.** A normal pirate victory already pays 200–400cr, which is many times what trading earns per trip. Bounties avoid adding to that, but the random fight rewards should be reviewed against the economy.

## Open Questions
- **Rewards.** Is replacing the 200–400cr payout with a smaller bounty a nerf players will resent? It keeps the economy sane, but it makes accepting a bounty pay less than fighting a random pirate.
- **Difficulty.** A single fixed raider means bounties get easy once the player has good equipment. Scaling the enemy by reward is the obvious later step.

## Acceptance Criteria
- [ ] Boards contain a mix of courier and bounty jobs; bounties are shown with their kind
- [x] Old saves load their jobs as courier jobs
- [ ] `accept`, `missions` and `abandon` work for bounty jobs
- [ ] Arriving at a bounty's system in time starts a pirate fight after travel events
- [ ] Victory pays the bounty reward (not the random payout), keeps loot and +5 reputation, and removes the job
- [ ] A successful escape, surrender and defeat fail the job for −3 reputation and apply the combat consequences
- [ ] Arriving late fails the job without a fight
- [ ] A simulation shows bounties out-earn courier jobs after repair costs, with a stated loss rate

## Testing (write first)
- [x] Board generation with a fixed seed: both kinds appear across many seeds, roughly 30% bounties
- [x] Bounty reward formula at known fuel costs
- [x] Old-save compatibility for jobs without a kind
- [ ] Settling a finished fight for each outcome (victory, flee, surrender, defeat) with injected dice
- [ ] Late arrival: job fails and no fight starts
- [ ] A failed flee attempt leaves the fight and the job running; a successful escape fails the job
- [ ] Two bounties for one system; escaping the first fails both
- [ ] Simulation over many seeded fights of net income per bounty

## Dependencies
- Feature 08: Combat ✅
- Feature 16: Jobs ✅
- Feature 12: Ship repair ✅ (repair is how the cost of a fight is paid)
