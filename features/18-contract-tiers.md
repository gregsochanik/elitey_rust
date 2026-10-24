# Feature 18: Contract Tiers and Criminal Contracts

## Overview
Jobs (Feature 16) and bounties (Feature 17) pay a flat amount whatever the player has become, so they stop mattering once trading scales. This feature gates harder, better paid contracts behind standing (Feature 10) and gives the player a second, darker career: criminal contracts that pay more and cost more.

The reputation score is one axis. Its two ends are two paths: a lawful commander gets lawful contracts, an infamous one gets criminal contracts, and a commander in the middle gets only the ordinary board.

## Status
In progress. Slices are tracked as GitHub issues; the access rules, freight, smuggling, the gateway job and standing-driven event odds are done; the balance pass remains.

## Version 1 Scope

### Who sees what
| Standing | Ordinary board | Lawful contracts | Criminal contracts |
|---|---|---|---|
| Respected, Trusted | yes | yes | no |
| Neutral | yes | no | gateway only |
| Disreputable, Criminal | yes | no | yes |

- Boards are rolled as today, with the extra kinds added to the pool for the commander's standing
- **Lawful contracts** are posted at systems with these governments: Democracy, Corporate State, Confederacy, Multi-Government, Communist, Dictatorship
- **Criminal contracts** are posted only at Anarchy and Feudal systems
- `jobs` labels each job's kind and shows why a job is there only through its kind (no hidden rules)

### Freight (lawful, Trusted and above)
- The client supplies N tons of a legal commodity (5 tons up to the ship's hold size) to carry to another system within range
- The cargo occupies the hold, so the player needs room to accept; it can be lost to events like any cargo
- **Reward:** the courier reward for the same distance × (1 + N / 20), so a bigger hold earns more (a 20-ton load pays about twice a courier job; a 60-ton Python load about four times)
- Deadline in jumps as for courier jobs; delivery pays and gives +2 reputation, failure or abandonment costs 3 reputation and the cargo

### Smuggling (criminal, Disreputable and below)
- The client supplies 5 to 10 tons of Narcotics to deliver to a lawful system within range
- The cargo occupies the hold, so acceptance needs that much free room; a full ship cannot accept the job
- **Reward:** about 3 times the courier reward for the same distance, plus a per-ton premium (exact numbers set by the balance pass)
- **Inspection on arrival:** smuggling has its own inspection roll when the player arrives at the destination; it does not use the random Government Inspection event, so the player faces at most one smuggling inspection per arrival, which covers every smuggling job for that system. The chance depends on the government (Communist and Dictatorship highest, Corporate State and Democracy next, the rest lowest)
- **Order on arrival:** travel events, then courier and freight deliveries, then the smuggling inspection, then bounty fights
- When inspected the player chooses:
  - **Bribe:** pay 40% of the combined reward of the smuggling jobs for this system and the deliveries succeed. Bribe is offered only if the player has the credits
  - **Run:** fight a patrol vessel (the same combat as a raider)
  - **Hand over the cargo:** the jobs fail
- Not inspected means every smuggling job here is paid in full

| Outcome | Jobs | Cargo, credits, hull and reputation |
|---|---|---|
| Not inspected, or bribed | Paid (minus the bribe) | Reputation −3 per job (infamy grows) |
| Run: victory | Paid in full | Reputation −3 per job and −5 for attacking the law; no combat payout or loot; the hull stays damaged |
| Run: successful escape | Fail | Contraband lost; the failure fine and reputation below apply; nothing else is lost |
| Run: surrender | Fail | Contraband lost; the failure fine and reputation below apply (the combat surrender losses are replaced by them) |
| Run: defeat | Fail | The combat defeat rules apply (all cargo lost, half the credits, hull at 10%, −5 reputation); the failure fine and extra reputation do not stack on top |
| Hand over the cargo | Fail | Contraband lost; the failure fine and reputation below apply |

- **Failure fine:** 25% of the job's reward, never more than the player's credits, and 5 reputation per failed job (once for all jobs lost to the same inspection)

### Gateway for Neutral players
- At Anarchy and Feudal systems, a Neutral commander is offered the occasional **black courier** job: a courier job whose parcel is contraband. It pays 1.5 times a courier job and costs 3 reputation on delivery
- Its purpose is to let a Neutral commander begin the criminal path without farming narcotics penalties

### Standing changes events
- Event odds already depend on the destination's government (Feature 06). Standing now adds to the weights: a Disreputable or Criminal commander meets more Military Patrol and Government Inspection events in lawful systems, and fewer in Anarchy and Feudal ones; a Trusted or Respected commander meets fewer
- Exact multipliers are set by the balance pass

### Safety rails
- Rescue (Feature 14), repair and refuel stay available to everyone, so a criminal is never locked into a dead end
- Abandoning a contract costs reputation as for other jobs

## Not in v1
- Assassination, spying and pirate lord contracts (see Later)
- A separate infamy score, or being lawful and criminal at once
- Rank (kills, jobs completed, net worth) as a second gate
- Bounty hunters sent after the player, wanted levels, bans from docking
- Reputation gating for the Neutral band beyond the gateway job

## Later
| Kind | Idea | Needs |
|---|---|---|
| Pirate lord (lawful) | A tougher raider for about 3 times a bounty | Enemy tiers in combat |
| Assassination (criminal) | Kill a named, stronger target | Enemy tiers; infamy cost |
| Spying (criminal) | Visit two systems in order, with inspection risk at each | Multi-leg jobs |
| Escort | Defend a convoy through an event | New combat mode |
| Rank gating | Skill, not just standing, opens harder jobs | Ranks and titles |
| Bounty hunters | Infamy brings fights to the player | Event weighting, combat |

## Balance
Contracts must pay well enough to be worth choosing over the patient trader (net about 84 per jump at 500cr, 151 at 2,000cr) and must carry real risk. Targets, set by simulation:
- Freight nets about 1.5 times a courier job at a 20-ton hold and scales with hold size
- Smuggling nets clearly more than freight when it succeeds and loses money when it fails; the break-even inspection chance should sit near the top of the government range
- A Criminal commander can still earn at least as much as a Neutral one by taking criminal contracts (so the path is a choice, not a penalty)

## Open Questions
- **One score or two.** v1 uses one axis. A commander who wants to be a respected trader with a dark side cannot be. A separate infamy score could allow it later.
- **Cargo supplied by clients.** Freight and smuggling put goods in the hold that the player did not buy. They must be marked so they cannot be sold at a market for profit.
- **Inspection choices.** Is Run too strong a default for a well-equipped ship? Tuning the patrol vessel against equipment tiers is likely needed.

## Acceptance Criteria
- [ ] The kinds of job on a board depend on the commander's standing and the system's government, as in the table
- [x] Freight jobs need room in the hold, put the cargo in it, and pay more for bigger loads
- [x] Freight delivery and failure follow the job rules, losing the cargo on failure
- [x] Smuggling jobs run an inspection on arrival with the three choices and the stated results, in the stated order after other deliveries
- [x] Smuggling needs free hold room to accept, and Bribe is only offered to a player who can pay
- [x] Contract cargo cannot be sold at a market
- [x] Neutral commanders are offered black courier jobs at Anarchy and Feudal systems and no other criminal jobs
- [x] Standing changes the odds of patrol and inspection events as specified
- [ ] Rescue, repair and refuel work for every standing
- [ ] Old saves load: jobs without the new kinds keep working
- [ ] A simulation sets the rewards and inspection odds within the balance targets

## Testing (write first)
- [ ] Board generation per standing and government: which kinds can and cannot appear, with a fixed seed
- [x] Freight: reward scales with tons, acceptance needs room, cargo is lost on failure
- [x] Smuggling: each inspection choice and each run outcome (victory, escape, surrender, defeat) with injected randomness and dice; fine capped by credits; penalties do not stack
- [x] Smuggling acceptance with a full hold; Bribe unavailable with too few credits
- [ ] Arrival order with a courier job, a freight job, a smuggling job and a bounty for the same system
- [x] Contract cargo cannot be sold
- [x] Event weights by standing
- [ ] Old-save compatibility
- [ ] Simulation: net income per jump for freight and smuggling against courier and trading

## Dependencies
- Feature 06: Dynamic Events ✅ (inspection, patrols, event weights)
- Feature 08: Combat ✅ (the patrol fight)
- Feature 10: Reputation ✅ (standings)
- Feature 14: Refuelling ✅ (rescue)
- Feature 16: Jobs ✅ and Feature 17: Bounty jobs ✅
- Feature 19: Market dynamics (not required, but both are needed to rebalance trading against jobs)
