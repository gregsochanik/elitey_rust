# Feature 04: Ship Upgrades

## Overview
Implement the ship upgrade system allowing players to improve their vessel with earned credits, enabling progression and expanded trading opportunities.

## Requirements

### Ship Types
- [x] **Cobra Mk III** (Starting ship)
  - Cargo: 20 tons
  - Fuel: 7 light years
  - Hull: 100%
  - Cost: 0 (starting ship)
- [x] **Cobra Mk IV** (First upgrade)
  - Cargo: 35 tons
  - Fuel: 10 light years
  - Hull: 120%
  - Cost: 500 credits
- [x] **Python** (Mid-game ship)
  - Cargo: 60 tons
  - Fuel: 12 light years
  - Hull: 150%
  - Cost: 2000 credits
- [x] **Anaconda** (End-game ship)
  - Cargo: 100 tons
  - Fuel: 15 light years
  - Hull: 200%
  - Cost: 5000 credits

### Upgrade Commands
- [x] `upgrade` - Show available ship upgrades
- [x] `upgrade <ship>` - Purchase ship upgrade
- [x] `ships` - List all available ships with stats

### Upgrade System
- [x] Ship comparison display
- [x] Credit requirement checking
- [x] Upgrade confirmation prompts
- [x] Ship transfer (cargo preservation)
- [x] Upgrade cost calculation

## Implementation Tasks

### 1. Ship Data Structure
- [x] Create data/ships.rs with ship definitions
- [x] Define ship statistics and capabilities
- [x] Implement ship comparison functions
- [x] Create ship upgrade paths

### 2. Upgrade Logic
- [x] Implement ship purchase validation
- [x] Create cargo transfer during upgrade
- [x] Add upgrade confirmation system
- [x] Implement ship statistics updates

### 3. Upgrade Commands
- [x] Implement `upgrade` command with ship list
- [x] Implement `upgrade <ship>` purchase logic
- [x] Implement `ships` command for ship comparison
- [x] Add upgrade status to status display

### 4. Ship Display
- [x] Create ship comparison display formatting
- [x] Show upgrade costs and benefits
- [x] Display current ship vs available upgrades
- [x] Add ship statistics to status display

## Upgrade Display Format
```
Available Upgrades
==================
Current Ship: Cobra Mk III
Credits: 750

Cobra Mk IV (500cr)
- Cargo: 20 → 35 tons (+15)
- Fuel: 7 → 10 light years (+3)
- Hull: 100% → 120% (+20%)
[upgrade] [cancel]

Python (2000cr) - Insufficient credits
- Cargo: 20 → 60 tons (+40)
- Fuel: 7 → 12 light years (+5)
- Hull: 100% → 150% (+50%)
```

## Acceptance Criteria
- [x] Ship upgrades are available when player has sufficient credits
- [x] Upgrade costs are correctly calculated and displayed
- [x] Cargo is preserved during ship transfers
- [x] Ship statistics are updated after upgrade
- [x] Cannot upgrade without sufficient credits
- [x] Upgrade confirmation prevents accidental purchases

## Testing
- [x] Test ship upgrade with sufficient credits
- [x] Test ship upgrade with insufficient credits
- [x] Test cargo preservation during upgrade
- [x] Test ship statistics accuracy after upgrade
- [x] Test upgrade confirmation system
- [x] Test ship comparison display

## Dependencies
- Feature 01: Core Gameplay (commander, credits, basic commands)
- Feature 03: Trading System (cargo management)

## Implementation Summary

**Status: ✅ COMPLETED**

Successfully implemented the ship upgrade system with the following components:

### Files Created/Modified:
- **`src/data/ships.rs`** - New ship data structures and upgrade logic
- **`src/game/upgrades.rs`** - New upgrade command handlers
- **`src/game/ship.rs`** - Updated to use ShipType enum and upgrade functionality
- **`src/game/game_state.rs`** - Integrated upgrade handler
- **`src/main.rs`** - Added upgrade and ships commands
- **`src/utils/display.rs`** - Updated help system with upgrade commands

### Key Features Implemented:
- Four ship types: Cobra Mk III (starting), Cobra Mk IV, Python, Anaconda
- Progressive upgrade system with increasing costs and capabilities
- Cargo preservation during ship upgrades
- Credit validation and upgrade cost calculation
- Rich display formatting with color-coded affordability indicators
- Comprehensive error handling and user feedback

### Commands Available:
- `upgrade` - Show available ship upgrades with costs and stats comparison
- `upgrade <ship>` - Purchase a specific ship upgrade (e.g., `upgrade cobra4`)
- `ships` - List all available ships with detailed specifications

The upgrade system provides clear progression paths for players to improve their trading capabilities as they earn credits through successful trading operations.
