# Feature 19: Market Dynamics

## Overview
Markets today re-roll every time the player arrives, and the player sees prices before committing. So a trader can carry cargo and hop between two systems until someone pays well, with almost no risk. In a simulation of that on Leesti and Riedquat (2.8 light years), a patient trader netted about 84 credits per jump at 500cr and 151 at 2,000cr, with 2% of trips losing money, beating courier jobs (53) and bounties (69). This feature makes markets remember and react to the player, so one route stops being a free loop.

## Status
In progress. Slices are tracked as GitHub issues; per-system market state, the world clock and clock-based refresh are done.

## Version 1 Scope

### Supersedes Feature 03
Feature 03 re-rolls fluctuation and refreshes stock on arrival elsewhere. From this feature on those rules are replaced: neither prices nor stock reset on arrival, only the world clock changes them. Feature 03 carries a note to that effect.

### Markets have memory
- Every system keeps its market between visits: base fluctuation, stock, and any price impact from the player
- The game counts jumps (a world clock). Markets change with that clock, not with arrivals

### Prices change by the clock
- A system's fluctuation (the ±30% around its economy's tendency) is fixed for a period of **10 jumps**, then re-rolled the next time it is looked at. Hopping between two systems does not re-roll anything
- Within a period, a system's undisturbed prices are the same on every visit, so a route can be learned and repeated. Prices the player has moved by trading stay moved and recover with the clock

### The player moves prices
- **Selling** lowers that commodity's sell price at the system: −1.5% per ton sold, down to −40%
- **Buying** raises that commodity's buy price: +1.5% per ton bought, up to +40%
- **Recovery:** each jump closes 10% of the gap between the current price and its undisturbed level, at every system, whether or not the player is there
- Buying still reduces stock; stock also recovers at 10% of the gap per jump toward its undisturbed level
- Standing's price modifier (Feature 10) applies after all this, as now

### Display
- `market` shows the same prices the player will get
- A commodity the player has pushed shows a marker: `▼ sold here` after selling, `▲ bought here` after buying. The marker clears once the remaining gap is under 20% of the original impact, which takes 16 jumps at the 10% recovery rate (0.9^16 ≈ 0.185)

### Saves
- The world clock and each system's market state are saved. Old saves load with a clock of zero and untouched markets

## Not in v1
- Prices that move with events (shortages, gluts) and jobs that react to them (supply jobs)
- Markets affecting each other, or NPC traders
- A price history screen
- Changes to the fluctuation range or the spread (Feature 03)

## Balance
Targets, set by simulation of an informed trader repeating one route with a full hold:
Simulation inputs: a Cobra Mk III with a 20-ton hold (Features 01 and 04), capital levels of 100, 500 and 2,000 credits, and market seeds 0 to 299. The clock advances with every jump, so "rests" means the player makes other jumps while the route recovers.

- The first trip pays about what it does today
- By the fifth trip the margin on that route has fallen to about the level of a courier job, and recovers while the player trades elsewhere
- A patient trader hopping between two systems earns no more than a bounty (about 70 per jump at 500cr)
- The informed single-trip numbers in Feature 03 stay valid for first visits

## Open Questions
- **Period length.** 10 jumps is a guess. Too short and hopping still works; too long and a bad market is stuck for a long time.
- **Impact size.** 1.5% per ton makes a 20-ton sale move a price by 30%. That may be too harsh for small holds or too soft for 100-ton ships. It may need to scale with the system's population.
- **Supply jobs.** Once markets move, a shortage can drive jobs. That is a later feature.

## Acceptance Criteria
- [ ] Arriving at a system does not change its prices; only the world clock does
- [x] A system's undisturbed prices and stock are identical on repeat visits within a 10-jump period and re-roll after it; arriving never resets either
- [ ] A pushed commodity's marker appears after trading and clears after 16 jumps
- [ ] Selling 20 tons visibly lowers the sell price at that system, and later visits show the lowered price recovering by 10% of the gap per jump
- [ ] Buying raises the buy price and lowers stock in the same way
- [ ] A simulation of repeated trips on one route shows the margin falling as in Balance
- [ ] The patient-trader strategy no longer beats bounties
- [x] The world clock and market state persist; old saves still load

## Testing (write first)
- [x] Prices and stock at a system are stable across arrivals within a period (when nothing is traded) and change across periods, with a fixed seed
- [ ] Marker appears after buying and selling and clears after 16 jumps
- [ ] Impact: price after selling N tons, the −40% and +40% caps, recovery per jump
- [ ] Stock depletion and recovery
- [ ] Standing modifier applied after impact, and `market`, `buy` and `sell` agree
- [x] Old-save compatibility
- [ ] Simulation: repeated-route income by trip number, and the patient-trader strategy

## Dependencies
- Feature 03: Trading ✅ (prices, stock)
- Feature 05: Save/Load ✅
- Feature 10: Reputation ✅ (price modifier)
- Feature 16: Jobs ✅ (the income this is balanced against)
