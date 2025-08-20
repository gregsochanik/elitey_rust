# Feature 03: Trading System

## Overview
Implement the core trading mechanics including commodities, market prices, buy/sell operations, and cargo management.

## Requirements

### Commodities System
- [ ] 8 different commodity types with base prices
- [ ] Commodity availability varies by system economy
- [ ] Price fluctuations based on supply/demand
- [ ] Illegal goods (narcotics) with higher risk/reward

### Market Commands
- [ ] `market` - Show current system's market prices
- [ ] `buy <item> <quantity>` - Purchase commodities
- [ ] `sell <item> <quantity>` - Sell commodities
- [ ] `cargo` - Show current cargo hold contents

### Price System
- [ ] Base prices for each commodity
- [ ] Economy-based price modifiers
- [ ] ±20% random price fluctuation
- [ ] Supply/demand effects on pricing
- [ ] Buy/sell price spread (typically 10-15%)

### Cargo Management
- [ ] Cargo capacity limits based on ship
- [ ] Cargo weight tracking
- [ ] Cannot exceed ship capacity
- [ ] Cargo display with quantities and values

## Implementation Tasks

### 1. Commodity Data Structure
- [ ] Create data/commodities.rs with commodity definitions
- [ ] Define base prices and economy preferences
- [ ] Implement price calculation algorithms
- [ ] Create commodity availability by system

### 2. Market System
- [ ] Implement market price generation
- [ ] Create buy/sell transaction logic
- [ ] Add price fluctuation algorithms
- [ ] Implement market display formatting

### 3. Cargo System
- [ ] Create cargo management structure
- [ ] Implement cargo capacity checking
- [ ] Add cargo display formatting
- [ ] Create cargo value calculations

### 4. Trading Commands
- [ ] Implement `market` command with price display
- [ ] Implement `buy` command with validation
- [ ] Implement `sell` command with validation
- [ ] Implement `cargo` command with detailed display

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
- [ ] Market prices are displayed correctly
- [ ] Buy/sell transactions work with proper validation
- [ ] Cargo capacity is enforced
- [ ] Price fluctuations are realistic
- [ ] Cargo display shows accurate information
- [ ] Invalid transactions are handled gracefully

## Testing
- [ ] Test buying commodities with sufficient credits
- [ ] Test buying commodities with insufficient credits
- [ ] Test selling commodities from cargo
- [ ] Test cargo capacity limits
- [ ] Test price fluctuation over time
- [ ] Test market display accuracy

## Dependencies
- Feature 01: Core Gameplay (commander, ship, basic commands) 
- Feature 02: Star Systems (system data, location tracking) 
