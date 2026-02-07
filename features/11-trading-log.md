# Feature 11: Trading Log

## Overview
Implement a comprehensive trading log system that tracks all buy/sell transactions, calculates profit/loss, and provides trading analytics to help players optimize their trading strategies.

## Requirements

### Core Trading Log Features
- [ ] `log` command to show trading history
- [ ] Track all buy/sell transactions with timestamps
- [ ] Show profit/loss calculations per trade
- [ ] Trading statistics and analytics
- [ ] Export trading data for analysis

### Transaction Tracking
- [ ] **Buy Transactions**: Commodity, quantity, price, location, timestamp
- [ ] **Sell Transactions**: Commodity, quantity, price, location, timestamp
- [ ] **Profit Calculation**: Sell price - Buy price per commodity
- [ ] **Route Analysis**: Track profitable routes and patterns
- [ ] **Performance Metrics**: Success rate, average profit, best trades

### Trading Analytics
- [ ] **Profit/Loss Summary**: Total earnings, losses, net profit
- [ ] **Best Trades**: Highest profit trades with details
- [ ] **Worst Trades**: Biggest losses for learning
- [ ] **Route Performance**: Most profitable system combinations
- [ ] **Commodity Analysis**: Best/worst performing commodities
- [ ] **Time Analysis**: Trading patterns over time

## Implementation Tasks

### 1. Transaction Logging
- [ ] Create transaction data structure
- [ ] Implement buy/sell transaction recording
- [ ] Add timestamp and location tracking
- [ ] Create transaction validation

### 2. Analytics Engine
- [ ] Implement profit/loss calculations
- [ ] Create trading statistics algorithms
- [ ] Add route analysis functionality
- [ ] Implement performance metrics

### 3. Display System
- [ ] Create trading log display formatting
- [ ] Implement analytics display
- [ ] Add export functionality
- [ ] Create detailed trade breakdowns

### 4. Integration
- [ ] Integrate with trading system
- [ ] Add to save/load system
- [ ] Update help and command system
- [ ] Add to status display

## User Experience

### Trading Log Display
```
Trading Log
===========
Recent Transactions:
- Bought 5 Food at Lave for 75cr
- Sold 5 Food at Leesti for 90cr (+15cr profit)
- Bought 3 Machinery at Leesti for 135cr
- Sold 3 Machinery at Lave for 150cr (+15cr profit)

Statistics:
- Total Trades: 24
- Total Profit: 450cr
- Best Trade: +25cr (Food, Lave→Leesti)
- Average Profit per Trade: 18.75cr
- Success Rate: 85%
```

### Detailed Trade Analysis
```
Trade Analysis: Food (Lave ↔ Leesti)
====================================
Buy Price: 15cr/ton
Sell Price: 18cr/ton
Profit Margin: 20%
Quantity Traded: 50 tons
Total Profit: 150cr
Route Frequency: 8 times
Success Rate: 100%
```

### Route Performance
```
Most Profitable Routes:
1. Lave ↔ Leesti (Food ↔ Machinery): +180cr
2. Industrial Loop: +120cr
3. Agricultural Circuit: +95cr

Best Commodities:
1. Food: +250cr total profit
2. Machinery: +180cr total profit
3. Textiles: +120cr total profit
```

## Acceptance Criteria
- [ ] All buy/sell transactions are logged accurately
- [ ] Profit/loss calculations are correct
- [ ] Trading statistics are meaningful and useful
- [ ] Log display is clear and informative
- [ ] Trading log persists across save/load
- [ ] Analytics help identify profitable patterns

## Testing
- [ ] Test transaction logging accuracy
- [ ] Test profit/loss calculations
- [ ] Test analytics algorithms
- [ ] Test log display formatting
- [ ] Test log persistence
- [ ] Test export functionality

## Dependencies
- Feature 01: Core Gameplay (commander, basic commands) ✅
- Feature 03: Trading System (buy/sell transactions) ✅
- Feature 05: Save/Load System (persistence) ✅
