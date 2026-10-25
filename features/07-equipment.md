# Feature 07: Equipment

Merges the former specs 07 (design) and 09 (implementation). Combat (08) depends on this.

## Overview
Players buy, install and remove ship equipment. Equipment takes slots and power, and later affects combat, travel and trading. Version 1 builds the foundation and the one effect that can be used today: cargo capacity.

## Status
Version 1 implemented. Effects other than cargo capacity are stored but inactive.

## Version 1 Scope
- [x] Equipment definitions (data below) with category, quality, price and power
- [x] Ship power and slot capacity per category (Cobra Mk III numbers for every ship; see Decisions)
- [x] Commander starts with the Basic item in each category (price 0, free), so a fresh ship matches today's behaviour
- [x] `equipment` lists items for sale at the current system
- [x] `equipment buy <item>` buys and installs; fails with a clear message if credits, slots or power are short
- [x] `equipment sell <item>` removes an installed item and refunds 50% of its price
- [x] `equipment installed` shows installed items, slots used and power used/available
- [x] Cargo racks change cargo capacity (Standard 1.0x, Expanded 1.25x, Military 1.5x, Elite 2.0x of the ship's base capacity)
- [x] Equipment is saved and loaded; old saves load with default equipment
- [x] Commands listed in `help`

Not in v1: weapon, shield, engine, sensor and special effects (stored but inactive until combat and travel use them), tech-level and government restrictions, damage and repair, upgrades.

## Equipment Data

### Weapons
| Equipment | Quality | Price | Power | Damage | Accuracy | Range |
|-----------|---------|-------|-------|--------|----------|-------|
| Pulse Laser | Basic | 50 | 2 | 15 | 0.9 | 1000 |
| Beam Laser | Standard | 120 | 4 | 25 | 0.8 | 1200 |
| Plasma Cannon | Military | 250 | 6 | 45 | 0.6 | 800 |
| Missile Launcher | Elite | 500 | 8 | 80 | 0.4 | 1500 |

### Shields
| Equipment | Quality | Price | Power | Protection | Recharge |
|-----------|---------|-------|-------|------------|----------|
| Basic Shield | Basic | 30 | 3 | 20 | 5 |
| Standard Shield | Standard | 80 | 5 | 40 | 8 |
| Military Shield | Military | 180 | 7 | 70 | 12 |
| Elite Shield | Elite | 350 | 10 | 100 | 15 |

### Engines
| Equipment | Quality | Price | Power | Speed | Fuel use |
|-----------|---------|-------|-------|-------|----------|
| Standard Drive | Basic | 20 | 2 | 1.0x | 1.0x |
| Performance Drive | Standard | 60 | 3 | 1.2x | 1.1x |
| Military Drive | Military | 150 | 5 | 1.4x | 1.2x |
| Elite Drive | Elite | 300 | 8 | 1.6x | 1.3x |

### Sensors
| Equipment | Quality | Price | Power | Range | Targeting |
|-----------|---------|-------|-------|-------|-----------|
| Basic Scanner | Basic | 15 | 1 | 1.0x | +0 |
| Advanced Scanner | Standard | 40 | 2 | 1.5x | +10 |
| Military Scanner | Military | 100 | 4 | 2.0x | +25 |
| Elite Scanner | Elite | 200 | 6 | 3.0x | +50 |

### Cargo Racks
| Equipment | Quality | Price | Power | Capacity |
|-----------|---------|-------|-------|----------|
| Standard Racks | Basic | 10 | 0 | 1.0x |
| Expanded Racks | Standard | 30 | 1 | 1.25x |
| Military Racks | Military | 80 | 2 | 1.5x |
| Elite Racks | Elite | 150 | 3 | 2.0x |

### Special
| Equipment | Quality | Price | Power | Effect |
|-----------|---------|-------|-------|--------|
| Heat Sink | Basic | 20 | 1 | −30% heat |
| Chaff Launcher | Standard | 50 | 2 | +40% evasion |
| ECM System | Military | 120 | 4 | +60% jamming |
| Cloaking Device | Elite | 250 | 8 | Temporary invisibility |

## Reactors
Power capacity comes from the installed reactor, a one-slot category that replaces like racks do. The starting Basic Reactor is free and gives 20 units, the Cobra Mk III's original capacity.

| Equipment | Quality | Price | Output |
|---|---|---|---|
| Basic Reactor | Basic | 10 | 20 |
| Improved Reactor | Standard | 60 | 28 |
| Military Reactor | Military | 160 | 38 |
| Elite Reactor | Elite | 350 | 50 |

- Reactors draw no power. A swap is refused if the installed equipment would need more than the new output, and the only reactor cannot be sold
- `status` shows `Power: used/capacity`; `equipment` and `equipment installed` show used, capacity and free power, with each item's draw
- Saves from before reactors load with the Basic Reactor

## Ship Capacity (Cobra Mk III)
- Slots: reactor 1, weapons 2, shields 1, engines 1, sensors 1, cargo 1, special 2

Cargo is one slot: racks replace each other, they don't stack. The earlier specs listed two cargo slots; one avoids stacking multipliers. Prices below are one tenth of the figures in the original specs, so upgrades are reachable from a 100cr start.

## Rules
- An item needs a free slot in its category and enough free power; otherwise the purchase fails and no credits are spent.
- Installed power cannot exceed the ship's capacity.
- In a single-slot category (shields, engines, sensors, cargo) buying replaces the installed item, which is refunded at half of what was paid. Weapons and special have two slots and refuse a third item.
- Starting equipment cost nothing, so it refunds nothing.
- Cargo capacity cannot drop below the tons currently held: swapping to a smaller rack is refused if the hold would be over capacity.

## Later Phases
- **Effects:** weapons and shields in combat (08), engines and sensors in travel, special items in combat.
- **Availability:** tech level (Military TL8+, Elite TL9+), economy (Elite only in rich economies) and government (weapons banned in communist systems) restrict what each system sells.
- **Condition:** damage during events, repair at stations.
- **Upgrades, quality and rarity, crafting:** descoped.

## Decisions
- **Prices** are one tenth of the original figures (items cost 10–500cr).
- **Other ships** use the Cobra Mk III's power and slots for now.
- **Changing ship** sells installed equipment at half of what was paid and resets to the starting loadout.
- **Item ids** are the lowercase name with dashes (`pulse-laser`); spaces and any case are accepted on the command line.

## Open Questions
- Per-ship power and slots, once ships differ in role.

## Acceptance Criteria
- [x] Buying installs the item and deducts credits
- [x] Purchases fail without spending credits when slots, power or credits are short
- [x] Selling refunds 50% and frees the slot and power
- [x] Cargo capacity follows the installed racks and never drops below cargo held
- [x] Equipment persists through save and load; old saves load with default equipment
- [x] A new game plays exactly as it did before this feature

## Testing (write first)
- [x] Install within slot and power limits; refuse over either limit
- [x] Failed purchase leaves credits and equipment unchanged
- [x] Sell refunds 50% and frees capacity
- [x] Cargo rack changes capacity; downgrade refused when cargo would not fit
- [x] Second cargo rack replaces the first
- [x] Save/load round trip keeps equipment; old save loads with defaults
- [x] `equipment installed` shows slots and power used

## Dependencies
- Feature 01: Core Gameplay ✅
- Feature 03: Trading ✅ (credits, cargo)
- Feature 04: Ship Upgrades ✅ (ship types)
- Feature 05: Save/Load ✅
