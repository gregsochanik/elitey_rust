# Feature 21: Locked Jobs

## Overview
The board hides contracts the commander is not allowed to take (Feature 18), so the player never learns they exist. Show them instead, marked as locked with the reason, so the player can see what standing or fighting skill would open.

## Status
Done (v1).

## Version 1 Scope
- The board lists every job the system posts, whatever the commander's standing. Which kinds a system posts still depends on its government: lawful systems post freight; lawless systems post smuggling and black courier jobs
- A job the commander cannot take is shown in the same numbered list, marked `[Locked: <reason>]`; the reward and route stay visible
- `accept` on a locked job refuses with the same reason and changes nothing
- Locks are checked against the commander's current standing and rating, so they change as the commander does

### Reasons
| Job | Needs | Locked when |
|---|---|---|
| Freight | Trusted or Respected | standing is Neutral or worse (reputation too low) |
| Smuggling | Disreputable or Criminal | standing is Neutral or better (reputation too high) |
| Black courier | Neutral | standing is better or worse than Neutral |
| Bounty | a combat rating by distance | rating is below it (combat rating too low) |
| Courier | nothing | never |

### Bounty ratings
| Fuel cost of the jump | Rating needed |
|---|---|
| 1 to 3 | Harmless (anyone) |
| 4 to 5 | Mostly Harmless |
| 6 and up | Poor |

Short bounties stay open to new commanders; longer, better paid ones are earned with kills (Feature 20).

## Balance
Bounties open to a Harmless commander (1 to 3 fuel) net slightly less than the best courier job after repairs (about 46 against 49 per jump), so bounties are the next step once the commander has some kills. `tests/economy_balance.rs` checks the 1.2 to 2 times margin for a commander rated Poor, who sees every bounty.

Lawless systems post black courier jobs on the top 30% of the roll, apart from smuggling's 40%, so a Neutral commander still sees them as often as before.

## Not in v1
- Ratings gating anything other than bounties
- Unlock hints beyond the lock reason, such as kills remaining

## Acceptance Criteria
- [x] A Neutral commander at a lawful system sees freight jobs, locked for standing
- [x] A Trusted commander at a lawless system sees black courier jobs, locked for standing
- [x] A Harmless commander sees long bounties locked for combat rating; short bounties open
- [x] `accept` on a locked job refuses with the reason and leaves the board and active jobs unchanged
- [x] A job unlocks as soon as standing or rating reaches the requirement
- [x] Boards from older saves still load; jobs without a rating requirement need none
- [x] Courier jobs are never locked

## Dependencies
- Feature 16: Jobs
- Feature 18: Contract tiers
- Feature 20: Combat rating
