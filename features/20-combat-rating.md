# Feature 20: Combat Rating

## Overview
In the original Elite a rating from Harmless to ELITE records how many ships the commander has destroyed. It sits beside reputation and measures something different: reputation is how the galaxy sees the commander, the combat rating is what they have done in fights. v1 only shows it; nothing is gated by it.

## Status
Done (v1).

## Version 1 Scope
- A commander counts **kills**: every enemy destroyed in a fight (pirate raiders, bounty targets, patrol vessels). Fleeing, surrendering and losing do not count
- Kills map to a rating:

| Kills | Rating |
|---|---|
| 0 | Harmless |
| 3 | Mostly Harmless |
| 6 | Poor |
| 12 | Average |
| 25 | Above Average |
| 50 | Competent |
| 100 | Dangerous |
| 200 | Deadly |
| 400 | ELITE |

- `status` shows `Combat rating: Poor (7 kills)` and how many kills the next rating needs; ELITE shows no next rating
- A promotion is announced when it happens, after the fight
- Kills are saved; older saves start at 0

## Not in v1
- Ratings gating jobs, equipment or encounters
- Weighting kills by how tough the enemy was
- Pirates or bounty hunters that react to the rating

## Acceptance Criteria
- [x] Winning a fight adds one kill; escaping, surrendering and defeat add none
- [x] Each threshold gives the rating in the table; promotions are announced
- [x] Kills from bounty fights and patrol-vessel fights count
- [x] `status` shows the rating, kills and the next target
- [x] Old saves load with 0 kills

## Dependencies
- Feature 08: Combat
- Feature 10: Reputation (shown beside it)
