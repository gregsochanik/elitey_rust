# Feature 16: Jobs

## Overview
Each system has a jobs board. The player can take paying jobs that give trips a purpose beyond buy-low-sell-high: deliver a parcel, haul a cargo, hunt a pirate. Version 1 is the smallest complete slice: **courier jobs only**, with a board, acceptance, deadlines and payment. Other job types come later and reuse the same board.

## Status
Done (v1). Board, courier jobs, accept/missions/abandon, delivery, deadlines and the balance pass are implemented.

## Why
Playing the game showed the loop has no goals beyond a 500cr ship, money rather than cargo space is the early limit, and a player with no credits has nothing to do. Jobs add short-term objectives, route planning against deadlines, and a way to earn without capital.

## Version 1 Scope

### The board
- `jobs` lists the jobs offered at the current system: 3 to 5, or every system in range when fewer than 3 are (SORI has only 2)
- Each shows a number, the destination, the reward, and the deadline in jumps
- The board is stable while the player stays in a system and is re-rolled on arrival elsewhere, like the market
- The board is generated from a seed, so tests can fix it
- Destinations are other systems within a full tank's range (7 light years) of the board's system

### Courier jobs
- A sealed parcel with no weight: it does not use cargo space and cannot be sold
- **Reward:** `20 + 7 × fuel cost of the direct jump`, varied by ±10%, rounded to whole credits (about 41cr for a 3-fuel jump, 69cr for a 7-fuel jump)
- **Deadline:** 2 to 4 jumps from accepting, counting every jump the player makes
- Paid in full on delivery; there is no advance

### Taking and tracking jobs
- `accept <n>` takes job *n* from the board; it leaves the board
- At most 3 jobs active at once
- `missions` lists active jobs numbered 1, 2, 3 with destination, reward and jumps left
- `abandon <n>` drops the active job with that number from `missions` at a cost of 3 reputation; numbers are the current list positions, so they change as jobs complete

### Delivery and failure
- Arriving at a job's destination with the job still in time completes it automatically: the reward is paid and reputation rises by 2
- Every jump uses one of the deadline. A job with deadline N must be delivered by the end of its Nth jump after accepting. If the Nth jump ends anywhere other than the destination, the job fails at once: it is removed and reputation falls by 3
- A job can be delivered on the jump that uses its last deadline jump
- Several jobs for the same destination all complete on arrival
- Delivery happens after any travel events on that jump

### Saves
- The board and active jobs are saved. Old saves without them load with an empty board and no active jobs (the board is generated on the next arrival or `jobs` call)

## Stranded relief
Done in Feature 14: a player with an empty hold and too little fuel to reach any system can call a rescue with a plain `refuel`, for 2 reputation. Jobs cannot fix that dead end alone, because even a courier job needs fuel to deliver.

## Not in v1
- Haul, bounty and supply jobs (see Later)
- Advance payments
- Standing gating the best jobs, or Criminal commanders being refused work
- Government flavour (lawful systems posting bounties, anarchies posting smuggling)
- Job chains, jobs that trigger events or ambushes
- Boards that change over time while docked

## Later
| Kind | Idea | Reuses |
|---|---|---|
| Haul | The client supplies N tons; deliver for a fee | Cargo hold |
| Bounty | Hunt a pirate raider at a named system | Combat (08) |
| Supply | Bring commodity X to a short-handed system at a premium | Markets (03) |
| Smuggling | Illegal goods from anarchies, with inspection risk | Events (06), reputation (10) |

## Balance
Compared net income per jump (reward minus fuel at 2cr per light year, picking the best job on the board) with an informed trader's net per one-way trip (Feature 03 method, 1,500 trips per capital level):

| | Net per jump or trip |
|---|---|
| Best courier job | 55 |
| Trading with 100cr | 12 |
| Trading with 300cr | 55 |
| Trading with 500cr | 96 |
| Trading with 1000cr | 157 |

Jobs pay about 4.5× trading at the starting 100cr, break even near 300cr, and fall behind beyond that. The first formula (`25 + 8 × fuel`) paid about 67 per jump, which would have paid for a 500cr ship in a handful of jobs, so it was lowered. Jobs carry no risk of loss, which is why they are allowed to out-earn trading early.

`tests/economy_balance.rs` keeps this: the best job earns at least twice a 100cr trader and no more than a 500cr trader.

## Open Questions
- **Parcel weight.** A weightless parcel is simplest. A 1-ton parcel would make full holds matter and interact with event cargo loss.
- **Abandoning.** With no advance there is nothing to refund, so abandoning only costs reputation. Revisit if advances arrive.
- **Board refresh.** Should the board change when the player stays but time passes? There is no clock, so v1 says no.
- **Reputation gating.** When reputation affects what is offered, a Criminal could be locked out of work. That worsens dead ends, and relies on the stranded rescue in Feature 14.

## Acceptance Criteria
- [x] `jobs` shows 3 to 5 courier jobs with number, destination, reward and deadline, all reachable on a full tank
- [x] The board is the same on repeat visits to the same screen and re-rolls on arrival elsewhere
- [x] `accept <n>` moves the job to `missions`; a fourth active job is refused
- [x] Delivering in time pays the reward, adds 2 reputation and clears the job
- [x] A job that runs out of jumps fails, is removed and costs 3 reputation
- [x] `abandon <n>` removes the job and costs 3 reputation
- [x] Jobs and the board persist across save and load; old saves still load
- [x] A simulation shows courier jobs out-earn trading at starting capital

## Testing (write first)
- [x] Board generation from a fixed seed: count, reachable destinations, reward range, deadline range
- [x] Reward formula at known fuel costs
- [x] Accept, the three-job limit, abandon
- [x] A job with deadline N: delivered on jump N succeeds; if jump N ends elsewhere it fails at once
- [x] Several jobs for one destination
- [ ] Delivery after an event on the jump
- [x] Save, load and old-save compatibility

## Dependencies
- Feature 02: Star Systems & Travel ✅ (jumps, fuel)
- Feature 05: Save/Load ✅
- Feature 14: Refuelling ✅ (the stranded rescue that jobs rely on)
- Feature 10: Reputation ✅ (rewards and penalties)
