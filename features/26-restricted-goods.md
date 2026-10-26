# Feature 26: Restricted Goods and Tech-Gated Equipment

## Overview
Not every market sells everything. Some commodities are only stocked under certain governments or at high enough tech, and equipment and ships need a minimum tech level at the shop. Locked entries stay visible with the reason, like locked jobs (Feature 21).

## Status
Commodities done (v1); equipment and ships in progress.

## Version 1 Scope

### Commodities
Rules live in `data/commodities.json` (`only_governments`, `not_governments`, `min_tech`).

| Commodity | Rule |
|---|---|
| Narcotics | sold only in Anarchy and Feudal systems |
| Luxury | not sold in Anarchy systems |
| Computers | sold only at tech level 6 or higher |

- The market lists every good; one that is not sold shows `not sold` and the reason
- Buying a good that is not sold is refused with the reason and changes nothing
- Selling is unaffected: any market buys any good (illegal goods still cost reputation)
- Contract cargo (smuggling jobs) does not come from the market, so it is unaffected

### Equipment and ships
- Each item and ship has a minimum tech level in its data file; the system's tech level decides what the shop and shipyard will sell
- Items above the local tech level are listed as locked with the reason (`needs tech level 8, this system is 5`); buying is refused
- Suggested levels: Basic 1, Standard 4, Military 8, Elite 11; Cobra Mk IV 4, Python 7, Anaconda 10

## Not in v1
- Restrictions by economy, and goods that appear only after a discovery (Feature 25 and issue #208)
