# Feature 16: Jobs

## Overview
Each system has a jobs board. The player can take paying jobs that give trips a purpose beyond buy-low-sell-high: deliver a parcel, haul a cargo, hunt a pirate. Version 1 is the smallest complete slice: **courier jobs only**, with a board, acceptance, deadlines and payment. Other job types come later and reuse the same board.

## Status
Spec only. Slices are tracked as GitHub issues.

## Why
Playing the game showed the loop has no goals beyond a 500cr ship, money rather than cargo space is the early limit, and a player with no credits has nothing to do. Jobs add short-term objectives, route planning against deadlines, and a way to earn without capital.

## Version 1 Scope

### The board
- `jobs` lists the jobs offered at the current system (3 to 5)
- Each shows a number, the destination, the reward, and the deadline in jumps
- The board is stable while the player stays in a system and is re-rolled on arrival elsewhere, like the market
- The board is generated from a seed, so tests can fix it
- Destinations are other systems within a full tank's range (7 light years) of the board's system

### Courier jobs
- A sealed parcel with no weight: it does not use cargo space and cannot be sold
- **Reward:** `25 + 8 × fuel cost of the direct jump`, varied by ±10%, rounded to whole credits (about 49cr for a 3-fuel jump)
- **Deadline:** 2 to 4 jumps from accepting, counting every jump the player makes
- Paid in full on delivery; there is no advance

### Taking and tracking jobs
- `accept <n>` takes job *n* from the board; it leaves the board
- At most 3 jobs active at once
- `missions` lists active jobs with destination, reward and jumps left
- `abandon <n>` drops an active job at a cost of 3 reputation

### Delivery and failure
- Arriving at a job's destination with the job still in time completes it automatically: the reward is paid and reputation rises by 2
- Every jump uses one of the deadline. A job with no jumps left that is not delivered on that jump fails: it is removed and reputation falls by 3
- A job can be delivered on the jump that uses its last deadline jump
- Several jobs for the same destination all complete on arrival
- Delivery happens after any travel events on that jump

### Saves
- The board and active jobs are saved. Old saves without them load with an empty board and no active jobs (the board is generated on the next arrival or `jobs` call)

## Stranded relief (separate bug fix)
The game has a dead end: a player with an empty hold, almost no credits and too little fuel to reach any system cannot do anything. Jobs cannot fix this alone because even a courier job needs fuel to deliver. Fix: when `refuel` finds the player stranded, a rescue crew tops up enough fuel to reach the nearest system for free, at a cost of 2 reputation.

- **Stranded** means: the hold is empty, and `fuel + credits / 2` (fuel affordable) is less than the fuel cost of the nearest system

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
A courier job should out-earn trading when the player is poor and fade as trading scales with capital. At the start an informed trader nets about 13cr per trip on 100cr, while a 3-fuel courier job pays about 49cr and costs about 6cr in fuel. At 500cr trading earns more per trip, so jobs stay useful for planning and rewards, not as the only income. Check this by simulation before tuning the formula.

## Open Questions
- **Parcel weight.** A weightless parcel is simplest. A 1-ton parcel would make full holds matter and interact with event cargo loss.
- **Abandoning.** With no advance there is nothing to refund, so abandoning only costs reputation. Revisit if advances arrive.
- **Board refresh.** Should the board change when the player stays but time passes? There is no clock, so v1 says no.
- **Reputation gating.** When reputation affects what is offered, a Criminal could be locked out of work. That worsens dead ends, so it needs the stranded relief first.

## Acceptance Criteria
- [ ] `jobs` shows 3 to 5 courier jobs with number, destination, reward and deadline, all reachable on a full tank
- [ ] The board is the same on repeat visits to the same screen and re-rolls on arrival elsewhere
- [ ] `accept <n>` moves the job to `missions`; a fourth active job is refused
- [ ] Delivering in time pays the reward, adds 2 reputation and clears the job
- [ ] A job that runs out of jumps fails, is removed and costs 3 reputation
- [ ] `abandon <n>` removes the job and costs 3 reputation
- [ ] Jobs and the board persist across save and load; old saves still load
- [ ] A stranded player who runs `refuel` is rescued once, for 2 reputation, and can jump again
- [ ] A simulation shows courier jobs out-earn trading at starting capital

## Testing (write first)
- [ ] Board generation from a fixed seed: count, reachable destinations, reward range, deadline range
- [ ] Reward formula at known fuel costs
- [ ] Accept, the three-job limit, abandon
- [ ] Deliver on the last allowed jump; fail one jump later
- [ ] Several jobs for one destination
- [ ] Delivery after an event on the jump
- [ ] Save, load and old-save compatibility
- [ ] Stranded detection and rescue; not triggered when the player can still afford to move

## Dependencies
- Feature 02: Star Systems & Travel ✅ (jumps, fuel)
- Feature 05: Save/Load ✅
- Feature 10: Reputation ✅ (rewards and penalties)
