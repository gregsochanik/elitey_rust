# Feature 04: Ship Upgrades

## Overview
Implement the ship upgrade system allowing players to improve their vessel with earned credits, enabling progression and expanded trading opportunities.

## Requirements

### Ship Types
- [ ] **Cobra Mk III** (Starting ship)
  - Cargo: 20 tons
  - Fuel: 7 light years
  - Hull: 100%
  - Cost: 0 (starting ship)
- [ ] **Cobra Mk IV** (First upgrade)
  - Cargo: 35 tons
  - Fuel: 10 light years
  - Hull: 120%
  - Cost: 500 credits
- [ ] **Python** (Mid-game ship)
  - Cargo: 60 tons
  - Fuel: 12 light years
  - Hull: 150%
  - Cost: 2000 credits
- [ ] **Anaconda** (End-game ship)
  - Cargo: 100 tons
  - Fuel: 15 light years
  - Hull: 200%
  - Cost: 5000 credits

### Upgrade Commands
- [ ] `upgrade` - Show available ship upgrades
- [ ] `upgrade <ship>` - Purchase ship upgrade
- [ ] `ships` - List all available ships with stats

### Upgrade System
- [ ] Ship comparison display
- [ ] Credit requirement checking
- [ ] Upgrade confirmation prompts
- [ ] Ship transfer (cargo preservation)
- [ ] Upgrade cost calculation

## Implementation Tasks

### 1. Ship Data Structure
- [ ] Create data/ships.rs with ship definitions
- [ ] Define ship statistics and capabilities
- [ ] Implement ship comparison functions
- [ ] Create ship upgrade paths

### 2. Upgrade Logic
- [ ] Implement ship purchase validation
- [ ] Create cargo transfer during upgrade
- [ ] Add upgrade confirmation system
- [ ] Implement ship statistics updates

### 3. Upgrade Commands
- [ ] Implement `upgrade` command with ship list
- [ ] Implement `upgrade <ship>` purchase logic
- [ ] Implement `ships` command for ship comparison
- [ ] Add upgrade status to status display

### 4. Ship Display
- [ ] Create ship comparison display formatting
- [ ] Show upgrade costs and benefits
- [ ] Display current ship vs available upgrades
- [ ] Add ship statistics to status display

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
- [ ] Ship upgrades are available when player has sufficient credits
- [ ] Upgrade costs are correctly calculated and displayed
- [ ] Cargo is preserved during ship transfers
- [ ] Ship statistics are updated after upgrade
- [ ] Cannot upgrade without sufficient credits
- [ ] Upgrade confirmation prevents accidental purchases

## Testing
- [ ] Test ship upgrade with sufficient credits
- [ ] Test ship upgrade with insufficient credits
- [ ] Test cargo preservation during upgrade
- [ ] Test ship statistics accuracy after upgrade
- [ ] Test upgrade confirmation system
- [ ] Test ship comparison display

## Dependencies
- Feature 01: Core Gameplay (commander, credits, basic commands)
- Feature 03: Trading System (cargo management)
