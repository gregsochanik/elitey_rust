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
- [x] Buy and sell each record: commodity, quantity, unit price, location, timestamp
- [x] Sells also record profit using average cost
- [x] Only successful transactions are recorded
- [x] Average cost is updated on each buy and cleared when holdings reach zero

### `log` Command
- [x] `log` shows the last 10 transactions, newest first
- [x] `log <n>` shows the last n transactions
- [x] Empty log shows a clear message
- [x] Listed in `help`

### Statistics (shown under the transaction list)
- [x] Total trades (buys + sells)
- [x] Net profit (sum of sell profits)
- [x] Best trade (highest single-sell profit)
- [x] Success rate (profitable sells ÷ sells)

### Persistence
- [x] Log is saved and loaded with the game
- [x] Saves created before this feature still load, with an empty log (`#[serde(default)]`)

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
- [x] Every successful buy and sell is logged accurately
- [x] Failed buys and sells (no credits, no space, no stock) are not logged
- [x] Profit matches average cost across multiple buys at different prices
- [x] Selling at a loss gives negative profit and lowers success rate
- [x] Log survives save then load
- [x] Old save files load without error

## Testing (write first)
- [x] Buy records a transaction
- [x] Sell records a transaction with correct profit
- [x] Average cost across two buys at different prices
- [x] Loss-making sale
- [x] Failed transactions leave the log unchanged
- [x] Statistics on an empty log and on a mixed log
- [x] Save/load round trip keeps the log
- [x] Loading a save without a log field gives an empty log

## Dependencies
- Feature 03: Trading System ✅
- Feature 05: Save/Load System ✅

## Future Work
Route performance, per-commodity analysis, time analysis (needs an in-game clock), and export.
