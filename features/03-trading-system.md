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
- [x] ±20% random price fluctuation
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
  - ±20% random price fluctuations
  - Buy/sell price spreads (10-15%)
  - Supply/demand based availability
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
