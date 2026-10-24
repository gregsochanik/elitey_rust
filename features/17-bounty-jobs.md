# Feature 17: Bounty Jobs

## Overview
A second job type for the board (Feature 16): hunt a pirate raider reported at another system. It is the first job that uses combat, so it pays more than a courier job and can cost the player their ship's health, their cargo and their credits.

## Status
Done (v1). Bounties are on boards, fought on arrival and balanced against courier jobs.

## Version 1 Scope

### On the board
- Each job on a board is a courier job (Feature 16) or, about 30% of the time, a bounty job
- A bounty job is listed as `Bounty: pirate raider near <system>` with the system, distance, fuel, reward and deadline in jumps
- Destinations, deadlines and numbering work exactly as for courier jobs; `accept`, `missions` and `abandon` handle both kinds (`missions` labels the kind)
- Saves written before bounties exist load their jobs as courier jobs

### Reward
- `80 + 10 × fuel cost of the direct jump`, varied by ±10%, rounded to whole credits (about 110cr for a 3-fuel jump, 150cr for a 7-fuel jump)
- Paid on victory **instead of** the random 100–200cr a pirate victory normally pays; the player still keeps the loot and the +5 reputation from combat

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
Net income per jump with the starting loadout, picking the best job on the board (reward minus fuel at 2cr per light year and repair at 2cr per hull point; a win costs about 29 hull, so about 58cr):

| | Net per jump |
|---|---|
| Best courier job | 53 |
| Best bounty, first formula (`100 + 12 × fuel`) | 100 |
| Best bounty, now (`80 + 10 × fuel`) | 69 |

The first formula paid nearly twice a courier job for a fight the starting ship wins every time, so it was lowered. At 69 a bounty nets about 1.3× a courier job before loot (1–3 tons of a random commodity) and the +5 reputation. The risk grows as equipment gets weaker: with no shield the raider wins about 14% of fights (Feature 08), and losing costs cargo, half the credits and a failed job.

`tests/economy_balance.rs` keeps this: the best bounty nets 1.2 to 2 times the best courier job. Trading with 500cr nets about 96 per trip and with 1,000cr about 157, so bounties are good for a poor player and fall behind once capital grows, like courier jobs.

**Separate issue, now done.** A random pirate victory pays 100–200cr (Feature 08), about the same as a bounty, so bounties give a guaranteed fight rather than more money.

## Open Questions
- **Rewards.** Does a bounty pay too little next to a random pirate victory (100–200cr plus loot)? A 3-fuel bounty pays about 110cr, so it is a guaranteed fight rather than more money; players may still expect more.
- **Difficulty.** A single fixed raider means bounties get easy once the player has good equipment. Scaling the enemy by reward is the obvious later step.

## Acceptance Criteria
- [x] Boards contain a mix of courier and bounty jobs; bounties are shown with their kind
- [x] Old saves load their jobs as courier jobs
- [x] `accept`, `missions` and `abandon` work for bounty jobs
- [x] Arriving at a bounty's system in time starts a pirate fight after travel events
- [x] Victory pays the bounty reward (not the random payout), keeps loot and +5 reputation, and removes the job
- [x] A successful escape, surrender and defeat fail the job for −3 reputation and apply the combat consequences
- [x] Arriving late fails the job without a fight
- [x] A simulation shows bounties out-earn courier jobs after repair costs, with a stated loss rate

## Testing (write first)
- [x] Board generation with a fixed seed: both kinds appear across many seeds, roughly 30% bounties
- [x] Bounty reward formula at known fuel costs
- [x] Old-save compatibility for jobs without a kind
- [x] Settling a finished fight for each outcome (victory, flee, surrender, defeat) with injected dice
- [x] Late arrival: job fails and no fight starts
- [x] A failed flee attempt leaves the fight and the job running; a successful escape fails the job
- [x] Two bounties for one system; escaping the first fails both
- [x] Simulation over many seeded fights of net income per bounty

## Dependencies
- Feature 08: Combat ✅
- Feature 16: Jobs ✅
- Feature 12: Ship repair ✅ (repair is how the cost of a fight is paid)
