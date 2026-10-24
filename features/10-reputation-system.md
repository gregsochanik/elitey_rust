# Feature 10: Reputation

## Overview
Reputation is one score for how the galaxy sees the commander. It already exists (events and combat change it, `status` shows it) but does nothing. Version 1 gives it names and two real effects: how illegal trade hurts it, and better or worse prices. Factions, per-system standing and bounties come later.

## Status
In progress. Slices are tracked as GitHub issues; standing, clamping and the narcotics penalty are done.

## Version 1 Scope

### The score
- A single integer on the commander, kept between −100 and 100 (changes clamp at the ends)
- Changes still come from events (−10 to +20) and combat (±5)

### Standing
The score maps to a named standing:

| Score | Standing | Price modifier |
|---|---|---|
| −100 to −30 | Criminal | −4% |
| −29 to −10 | Disreputable | −2% |
| −9 to 9 | Neutral | 0% |
| 10 to 29 | Trusted | +2% |
| 30 to 100 | Respected | +4% |

- `status` shows `Reputation: 12 (Trusted)`
- A change in standing is announced when it happens

### Illegal trade
- Buying or selling Narcotics costs 2 reputation per transaction (not per ton)
- The message says why

### Prices
- A good standing improves both sides of a trade: the player pays less when buying and receives more when selling, by the modifier above. A bad standing does the opposite
- The modifier applies to the price the player actually pays or receives; the `market` display shows the modified prices and the standing that caused them
- The modifier is the same in every system
- One function does the arithmetic for `market`, `buy` and `sell`. It runs last, on the price Feature 03 already produces (economy, fluctuation, spread, stock): `final = round(price × (1 + modifier))`, rounded half up to whole credits and never below 1

## Not in v1
- Factions and government-specific standing
- Per-system reputation
- Bounties and a separate criminal record
- Reputation decay over time
- Equipment access, system access, new event choices (see Open Questions)

## Balance
A 4% bonus is about the size of the buy/sell spread (2–6%), so a Respected trader roughly doubles the margin on a typical trade, and a Criminal trader loses most of it. Check this by simulation before tuning the numbers.

## Open Questions
- **Event outcomes.** Should standing change event odds or choices (for example, inspections going easier for the Trusted)? Left out of v1 to keep the first slice about prices.
- **Equipment access.** The original spec gated elite equipment on reputation. That needs tech levels first (Feature 07, later).
- **Recovery.** With no decay, a Criminal commander can only recover through events and combat victories. Is that enough?

## Acceptance Criteria
- [x] The score never leaves −100 to 100
- [x] `status` shows the standing next to the score, and standing changes are announced
- [x] Trading Narcotics lowers reputation by 2 per transaction and says so
- [ ] A Respected commander pays less and sells for more than a Neutral one; a Criminal pays more and sells for less
- [ ] `market` shows the prices the player will actually get
- [ ] Reputation and its effects persist across save and load
- [ ] A simulation keeps the price effect in a sensible range

## Testing (write first)
- [x] Standing for every boundary score (−100, −30, −29, −10, −9, 9, 10, 29, 30, 100)
- [x] Clamping at both ends
- [ ] Price modifier applied to buy and to sell with the rule above; `market`, `buy` and `sell` agree on the same price
- [x] Narcotics penalty on buy and on sell; other goods unaffected
- [ ] Old saves without a reputation score load as 0 (Neutral); standing and price modifier are derived from the score after loading, never stored
- [ ] Simulation: informed trader earnings by standing

## Dependencies
- Feature 03: Trading ✅
- Feature 06: Dynamic Events ✅
- Feature 08: Combat ✅
