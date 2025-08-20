# Feature 09: Equipment System

## Overview
The equipment system allows players to purchase, install, and manage ship equipment that provides various bonuses and capabilities. Equipment affects combat, travel, trading, and ship performance.

## Current Implementation Status

**✅ Fully Implemented:**
- Equipment data structure and definitions
- Power/slot system with ship capacity limits
- Equipment shop inventory generation based on system characteristics
- Equipment installation and removal with validation
- Equipment status display with power usage
- Equipment damage and repair system
- Equipment shop display with availability filtering

**❌ Missing Features:**
- `equipment buy <item_id>` - Purchase equipment ✅
- `equipment sell <item_id>` - Sell equipment ✅
- `equipment installed` - View installed equipment ✅

## Equipment Categories

### 🗡️ Weapons
| Equipment | Quality | Price | Power | Damage | Accuracy | Range |
|-----------|---------|-------|-------|--------|----------|-------|
| Pulse Laser | Basic | 500 | 2 | 15 | 0.9 | 1000 |
| Beam Laser | Standard | 1200 | 4 | 25 | 0.8 | 1200 |
| Plasma Cannon | Military | 2500 | 6 | 45 | 0.6 | 800 |
| Missile Launcher | Elite | 5000 | 8 | 80 | 0.4 | 1500 |

### 🛡️ Shields
| Equipment | Quality | Price | Power | Protection | Recharge |
|-----------|---------|-------|-------|------------|----------|
| Basic Shield | Basic | 300 | 3 | 20 | 5 |
| Standard Shield | Standard | 800 | 5 | 40 | 8 |
| Military Shield | Military | 1800 | 7 | 70 | 12 |
| Elite Shield | Elite | 3500 | 10 | 100 | 15 |

### 🚀 Engines
| Equipment | Quality | Price | Power | Speed | Fuel |
|-----------|---------|-------|-------|-------|------|
| Standard Drive | Basic | 200 | 2 | 1.0x | 1.0x |
| Performance Drive | Standard | 600 | 3 | 1.2x | 1.1x |
| Military Drive | Military | 1500 | 5 | 1.4x | 1.2x |
| Elite Drive | Elite | 3000 | 8 | 1.6x | 1.3x |

### 📡 Sensors
| Equipment | Quality | Price | Power | Range | Targeting |
|-----------|---------|-------|-------|-------|-----------|
| Basic Scanner | Basic | 150 | 1 | 1.0x | +0 |
| Advanced Scanner | Standard | 400 | 2 | 1.5x | +10 |
| Military Scanner | Military | 1000 | 4 | 2.0x | +25 |
| Elite Scanner | Elite | 2000 | 6 | 3.0x | +50 |

### 📦 Cargo Equipment
| Equipment | Quality | Price | Power | Capacity |
|-----------|---------|-------|-------|----------|
| Standard Racks | Basic | 100 | 0 | 1.0x |
| Expanded Racks | Standard | 300 | 1 | 1.25x |
| Military Racks | Military | 800 | 2 | 1.5x |
| Elite Racks | Elite | 1500 | 3 | 2.0x |

### 🔧 Special Equipment
| Equipment | Quality | Price | Power | Effect |
|-----------|---------|-------|-------|--------|
| Heat Sink | Basic | 200 | 1 | -30% heat |
| Chaff Launcher | Standard | 500 | 2 | +40% evasion |
| ECM System | Military | 1200 | 4 | +60% jamming |
| Cloaking Device | Elite | 2500 | 8 | 30s invisibility |

## Ship Capacity (Cobra Mk III)

### Power & Slots
- **Power Capacity**: 20 units
- **Weapon Slots**: 2
- **Shield Slots**: 1
- **Engine Slots**: 1
- **Sensor Slots**: 1
- **Cargo Slots**: 2
- **Special Slots**: 2

### Installation Constraints
- Equipment requires specific power units
- Equipment requires specific slot types
- System prevents overloading power or slots
- Cannot install more equipment than capacity allows

## Shop System

### Availability Logic
- **Tech Level Restrictions**:
  - Military equipment: Requires TL8+
  - Elite equipment: Requires TL9+
- **Economy Restrictions**:
  - Elite equipment: Only available in rich economies
- **Government Restrictions**:
  - Weapons: Banned in communist systems

### System-Based Inventory
- Different equipment available at different stations
- Based on system economy, government, and tech level
- Dynamic inventory generation

## Equipment Effects (Planned)

### Combat Effects
- **Weapons**: Damage, accuracy, range bonuses
- **Shields**: Protection, recharge rate
- **Special**: Evasion, jamming, cloaking

### Travel Effects
- **Engines**: Speed multipliers, fuel efficiency
- **Sensors**: Range detection, targeting bonuses

### Trading Effects
- **Cargo**: Capacity multipliers
- **Special**: Market access, smuggling bonuses

## Implementation Tasks

### Phase 1: Complete User Commands
- [ ] Implement `equipment buy <item_id>` command
- [ ] Implement `equipment sell <item_id>` command
- [ ] Implement `equipment installed` command
- [ ] Add equipment commands to help menu
- [ ] Add equipment commands to input validation

### Phase 2: Equipment Effects Integration
- [ ] Integrate weapon effects in combat system
- [ ] Integrate shield effects in combat system
- [ ] Integrate engine effects in travel system
- [ ] Integrate sensor effects in travel system
- [ ] Integrate cargo effects in trading system
- [ ] Add equipment effects display command
- [ ] Update cargo display to show equipment bonuses
- [ ] Add equipment effects to help menu

### Phase 3: Advanced Features DESCOPED
- [ ] Equipment condition system (damage/repair)
- [ ] Equipment upgrade system
- [ ] Equipment specialization bonuses

## Technical Requirements

### Files to Modify
- `game/game_state.rs` - Add equipment commands
- `utils/input.rs` - Add command validation
- `utils/display.rs` - Add equipment display functions
- `game/combat.rs` - Integrate weapon/shield effects
- `game/travel.rs` - Integrate engine/sensor effects
- `game/trading.rs` - Integrate cargo effects

## Notes
- Equipment system provides significant depth to ship customization
- Power/slot system creates meaningful choices
- System-based availability adds variety to different stations
- Equipment effects will significantly impact gameplay 