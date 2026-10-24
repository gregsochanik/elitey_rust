# Feature 03: Trading System

## Overview
Implement the core trading mechanics including commodities, market prices, buy/sell operations, and cargo management.

## Requirements

### Commodities System
- [x] 8 different commodity types with base prices
- [x] Commodity availability varies by system economy
- [x] Price fluctuations based on supply/demand
- [x] Illegal goods (narcotics) with higher risk/reward

### Market Commands
- [x] `market` - Show current system's market prices
- [x] `buy <item> <quantity>` - Purchase commodities
- [x] `sell <item> <quantity>` - Sell commodities
- [x] `cargo` - Show current cargo hold contents

### Price System
- [x] Base prices for each commodity
- [x] Economy-based price modifiers
- [x] ±30% random price fluctuation
- [x] Supply/demand effects on pricing
- [x] Buy/sell price spread (typically 10-15%)

### Cargo Management
- [x] Cargo capacity limits based on ship
- [x] Cargo weight tracking
- [x] Cannot exceed ship capacity
- [x] Cargo display with quantities and values

## Implementation Tasks

### 1. Commodity Data Structure
- [x] Create data/commodities.rs with commodity definitions
- [x] Define base prices and economy preferences
- [x] Implement price calculation algorithms
- [x] Create commodity availability by system

### 2. Market System
- [x] Implement market price generation
- [x] Create buy/sell transaction logic
- [x] Add price fluctuation algorithms
- [x] Implement market display formatting

### 3. Cargo System
- [x] Create cargo management structure
- [x] Implement cargo capacity checking
- [x] Add cargo display formatting
- [x] Create cargo value calculations

### 4. Trading Commands
- [x] Implement `market` command with price display
- [x] Implement `buy` command with validation
- [x] Implement `sell` command with validation
- [x] Implement `cargo` command with detailed display

## Market Display Format
```
Lave Market
============
Food:      Buy: 15cr  Sell: 12cr  Available: 50 tons
Textiles:  Buy: 25cr  Sell: 22cr  Available: 30 tons
Machinery: Buy: 45cr  Sell: 42cr  Available: 15 tons
...
```

## Cargo Display Format
```
Cargo Hold (12/20 tons)
=======================
Food:      5 tons  Value: 60cr
Machinery: 2 tons  Value: 84cr
Textiles:  5 tons  Value: 110cr
Total Value: 254cr
```

## Acceptance Criteria
- [x] Market prices are displayed correctly
- [x] Buy/sell transactions work with proper validation
- [x] Cargo capacity is enforced
- [x] Price fluctuations are realistic
- [x] Cargo display shows accurate information
- [x] Invalid transactions are handled gracefully

## Testing
- [x] Test buying commodities with sufficient credits
- [x] Test buying commodities with insufficient credits
- [x] Test selling commodities from cargo
- [x] Test cargo capacity limits
- [x] Test price fluctuation over time
- [x] Test market display accuracy

## Dependencies
- Feature 01: Core Gameplay (commander, ship, basic commands) 
- Feature 02: Star Systems (system data, location tracking)

## Implementation Status: ✅ COMPLETED

### Summary
Feature 03: Trading System has been successfully implemented with all requirements met. The implementation includes:

- **Complete commodities system** with 8 different goods and economy-based pricing
- **Dynamic market generation** with price fluctuations and supply/demand effects
- **Trading commands** (`market`, `buy`, `sell`, `cargo`) with comprehensive validation
- **Cargo management** with capacity limits and detailed display
- **Economy-based pricing** that varies by system type and includes buy/sell spreads
- **Comprehensive test suite** covering all trading scenarios and edge cases

### Files Created/Modified
- `src/data/commodities.rs` - Commodity definitions, pricing, and cargo management
- `src/data/commodities.rs` - Comprehensive tests for trading functionality
- `src/game/trading.rs` - Trading command handlers and market logic
- `src/game/trading.rs` - Trading system tests
- `src/game/ship.rs` - Updated to use new cargo system
- `src/game/game_state.rs` - Added trading handler integration
- `src/main.rs` - Added trading commands to main game loop
- `src/utils/display.rs` - Added trading-related display functions

### Key Features Implemented
- **Commodities**: Food, Textiles, Machinery, Computers, Minerals, Alloys, Luxury, Narcotics
- **Market Mechanics**: 
  - Economy-based price modifiers (agricultural systems prefer food/textiles, industrial prefer machinery/minerals)
  - ±30% random price fluctuations, fixed while the player stays in a system and re-rolled on arrival elsewhere
  - Buy/sell price spreads (2-6%)
  - Supply/demand based availability; buying reduces stock for the rest of the visit, and stock refreshes on arrival elsewhere
- **Trading Commands**: 
  - `market` - Shows current system's prices and availability
  - `buy <item> <quantity>` - Purchases commodities with validation
  - `sell <item> <quantity>` - Sells commodities with validation
  - `cargo` - Shows cargo contents with values
- **Cargo Management**: Capacity limits, weight tracking, and detailed display

### Ready for Next Features
The trading system foundation is now ready for implementing:
- Feature 04: Ship Upgrades
- Feature 05: Save/Load System
- Feature 06: Dynamic Events
- And all subsequent features

**Status: ✅ PRODUCTION READY** 

## Balancing: Early-Game Profit

**Problem:** Trading is rarely profitable at the start. Sampling six LAVE→DISO trips, the best per-ton margin ranged from −2cr to +10cr, and a mixed run took 10 attempts to make a profit. With 100cr starting capital, one bad trade is a large loss.

**Cause:** The sell price is 85–95% of the buy price in the same system (an average 10% loss), while per-system price differences are only ±20% random. The spread eats most of the arbitrage margin.

### Tasks
- [x] Tighten the buy/sell spread (sell at 94–98% of buy, was 85–95%)
- [x] Add a simulation test: across many seeds, the best commodity on a LAVE→DISO trip makes a profit in at least 90% of runs (measured 94%, was 80%)
- [x] Re-check starting credits (100cr) and cargo capacity (20 tons) after the spread change
- [x] Simulate round trips and record the typical profit per trip (see Fuel and the early game)

### Acceptance Criteria
- [ ] A player choosing the best commodity usually profits on a short trip
- [ ] A poor choice (or bad luck) still loses money, so choices matter
- [ ] Prices remain stable within a system visit

### Result and follow-up
The spread change raised the hindsight measure (best commodity profits on a trip) from 80% to 94%, and the simulation test now asserts at least 90%.

**Limitation:** that measure is hindsight. A player buys before seeing the destination's prices, and prices re-roll on arrival. Play-testing 15 blind LAVE→DISO trips (one each of Food, Textiles and Minerals) lost money in 13 of them, typically 1–18cr. A random purchase has an expected loss of about the spread, so trading is still closer to gambling than skill.

- [x] Give players information to trade on, so "buy low, sell high" is possible. Options: show each price against the commodity's base price in `market`, show the economy-based tendency (cheap/expensive) per commodity, or let `info <system>` hint at what a system buys and sells
- [x] Re-run the blind-trip play-test afterwards; informed picks should usually profit and blind picks should not

**Done:** `market` now shows each buy price as a percentage against the commodity's base price (green at −10% or lower, red at +10% or higher). Prices at the destination are independent of the origin, so buying low against base is a real edge.

| Strategy (LAVE→DISO, 1 ton) | Profitable | Average profit |
|---|---|---|
| Buy the cheapest vs base | 14 of 23 (61%) | +5.7cr |
| Buy blind | 4 of 24 (17%) | −2.0cr |

A simulation test over four route pairs agrees: 62% for informed picks against 39% for blind ones.

## Fuel and the early game

Fuel (2cr per light year) and repair (2cr per point) are now real costs. With the old ±20% price swing an informed trader on 100cr made about 8cr gross per trip, and 5cr fuel made trading a net loss.

| Price swing | Fuel/ly | Gross per trip (100cr) | Net per trip | Trips that lose money |
|---|---|---|---|---|
| ±20% | 0 | 8 | 8 | 30% |
| ±20% | 5 | 8 | −9 | 73% |
| ±30% | 2 | 20 | 13 | 32% |
| ±40% | 2 | 30 | 23 | 25% |

Chosen: ±30% swing and 2cr per light year. About 13cr net per trip at the start, growing with capital, so a 500cr ship upgrade takes roughly 15–20 trips. Trips still lose money about a third of the time.

`tests/economy_balance.rs` keeps this: an informed trader's gross profit is at least twice the fuel cost and at least 10% of the starting credits, and between 5% and 40% of trips still lose money.
