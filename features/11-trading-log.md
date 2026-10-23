# Feature 11: Trading Log

## Overview
Record every buy and sell, calculate profit per sale, and show recent trades and totals via a `log` command. The log persists with the save file.

Out of scope for this version (later): route analysis, per-commodity analytics, time analysis, export.

## Design Decisions
- **Profit uses average cost.** The cargo hold stores only quantities, so the log tracks a running average cost per commodity. Profit on a sale = `(sell price − average cost) × quantity`.
- **Cargo gained or lost outside trading** (events, ship changes) is not logged. Gained cargo has no cost basis, so it counts as cost 0; lost cargo reduces held quantity but leaves average cost unchanged.
- **Timestamps are real-world** (Unix seconds). There is no in-game clock.
- **Profit is signed** (`i64`); credits stay `u32`.
- **A trade is one buy or sell transaction.** Success rate = profitable sells ÷ total sells.

## Requirements

### Transaction Recording
- [ ] Buy and sell each record: commodity, quantity, unit price, location, timestamp
- [ ] Sells also record profit using average cost
- [ ] Only successful transactions are recorded
- [ ] Average cost is updated on each buy and cleared when holdings reach zero

### `log` Command
- [ ] `log` shows the last 10 transactions, newest first
- [ ] `log <n>` shows the last n transactions
- [ ] Empty log shows a clear message
- [ ] Listed in `help`

### Statistics (shown under the transaction list)
- [ ] Total trades (buys + sells)
- [ ] Net profit (sum of sell profits)
- [ ] Best trade (highest single-sell profit)
- [ ] Success rate (profitable sells ÷ sells)

### Persistence
- [ ] Log is saved and loaded with the game
- [ ] Saves created before this feature still load, with an empty log (`#[serde(default)]`)

## Example Output
```
Trading Log
===========
Recent Transactions:
- Sold 5 Food at Leesti for 90cr (+15cr profit)
- Bought 5 Food at Lave for 75cr

Statistics:
- Total Trades: 2
- Net Profit: 15cr
- Best Trade: +15cr (5 Food at Leesti)
- Success Rate: 100%
```

## Acceptance Criteria
- [ ] Every successful buy and sell is logged accurately
- [ ] Failed buys and sells (no credits, no space, no stock) are not logged
- [ ] Profit matches average cost across multiple buys at different prices
- [ ] Selling at a loss gives negative profit and lowers success rate
- [ ] Log survives save then load
- [ ] Old save files load without error

## Testing (write first)
- [ ] Buy records a transaction
- [ ] Sell records a transaction with correct profit
- [ ] Average cost across two buys at different prices
- [ ] Loss-making sale
- [ ] Failed transactions leave the log unchanged
- [ ] Statistics on an empty log and on a mixed log
- [ ] Save/load round trip keeps the log
- [ ] Loading a save without a log field gives an empty log

## Dependencies
- Feature 03: Trading System ✅
- Feature 05: Save/Load System ✅

## Future Work
Route performance, per-commodity analysis, time analysis (needs an in-game clock), and export.
