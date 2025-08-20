# Feature 10: Reputation System 

## Overview
Implement a reputation system that tracks player standing with different systems and factions, affecting trading prices, event outcomes, and access to certain equipment and services.

## Requirements

### Core Reputation Mechanics
- [ ] Trading reputation affects prices (buy/sell spreads)
- [ ] Criminal record for illegal trading activities
- [ ] Faction relations with different systems
- [ ] Reputation display in status command
- [ ] Reputation decay over time

### Reputation Types
- [ ] **Trading Reputation**: Affects market prices and availability
- [ ] **Criminal Record**: Tracks illegal activities and consequences
- [ ] **Faction Relations**: Standing with different government types
- [ ] **System Reputation**: Individual system standing

### Reputation Effects
- [ ] **Price Modifiers**: Better reputation = better prices
- [ ] **Equipment Access**: Elite equipment requires good reputation
- [ ] **Event Outcomes**: Reputation affects event choices and results
- [ ] **System Access**: Some systems require minimum reputation
- [ ] **Bounty System**: Criminal activities result in bounties

## Implementation Tasks

### 1. Reputation Tracking
- [ ] Create reputation data structure
- [ ] Implement reputation calculation algorithms
- [ ] Add reputation persistence to save system
- [ ] Create reputation display functions

### 2. Trading Integration
- [ ] Integrate reputation with price calculations
- [ ] Add reputation effects to market display
- [ ] Implement criminal record for illegal goods
- [ ] Create reputation-based equipment restrictions

### 3. Event Integration
- [ ] Add reputation effects to event outcomes
- [ ] Implement reputation-based event choices
- [ ] Create reputation consequences for event actions
- [ ] Add reputation gains/losses from events

### 4. UI and Display
- [ ] Add reputation to status display
- [ ] Create detailed reputation breakdown
- [ ] Add reputation history tracking
- [ ] Implement reputation change notifications

## Files Created/Modified
```
game/
├── reputation.rs        # Reputation tracking system ✅
├── trading.rs           # Add reputation to price calculations ✅
├── events.rs            # Add reputation effects to events ✅
└── game_state.rs        # Add reputation to status display ✅
utils/
├── display.rs           # Add reputation display functions ✅
└── serialization.rs     # Add reputation to save system ✅
data/
└── factions.rs          # Faction definitions and relations ✅
```

## Reputation Data Structure
```lua
reputation = {
    trading = {
        lave = 75,        -- Good trading reputation
        leesti = 45,      -- Neutral reputation
        isditixe = -20    -- Poor reputation (criminal)
    },
    criminal_record = {
        illegal_trades = 3,
        total_bounties = 150,
        wanted_systems = {"isditixe", "corporate_system"}
    },
    faction_relations = {
        agricultural = 60,
        industrial = 30,
        corporate = -10,
        communist = 80
    },
    system_access = {
        elite_equipment = true,
        restricted_systems = false,
        special_events = true
    }
}
```

## User Experience

### Status Display
```
Commander: Greg
Credits: 1500
Location: Leesti
Ship: Cobra Mk III
Cargo: 8/20 tons
Fuel: 6/7 light years
Hull: 95%

Reputation:
- Trading: Good (Leesti: +15% prices)
- Criminal: Clean record
- Factions: Agricultural (+60), Industrial (+30)
- Access: Elite equipment available
```

### Reputation Effects
```
Market at Leesti (Good Reputation)
===============================
Food:      Buy: 16cr  Sell: 14cr  (+10% from reputation)
Machinery: Buy: 49cr  Sell: 46cr  (+10% from reputation)
```

### Criminal Consequences
```
⚠️  WARNING: Illegal goods detected!
Criminal record updated: +1 illegal trade
Bounty added: 50 credits
Reputation affected: -10 in corporate systems
```

## Acceptance Criteria
- [ ] Reputation affects market prices appropriately
- [ ] Criminal activities are tracked and have consequences
- [ ] Reputation is displayed in status command
- [ ] Reputation affects event outcomes
- [ ] Reputation persists across save/load
- [ ] Reputation changes are clearly communicated

## Testing
- [ ] Test reputation effects on trading prices
- [ ] Test criminal record tracking
- [ ] Test reputation display accuracy
- [ ] Test reputation effects on events
- [ ] Test reputation persistence
- [ ] Test reputation-based restrictions

## Dependencies
- Feature 01: Core Gameplay (commander, basic commands) ✅
- Feature 03: Trading System (market, cargo) ✅
- Feature 06: Dynamic Events (event outcomes) ✅
- Feature 05: Save/Load System (persistence) ✅

## Estimated Effort
- **Reputation Tracking**: 3-4 hours ✅
- **Trading Integration**: 2-3 hours ✅
- **Event Integration**: 2-3 hours ✅
- **UI and Display**: 2-3 hours ✅
- **Testing**: 2-3 hours ✅
- **Total**: 11-16 hours ✅ **COMPLETED** 