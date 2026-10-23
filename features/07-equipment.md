# Feature 07: Equipment

Merges the former specs 07 (design) and 09 (implementation). Combat (08) depends on this.

## Overview
Players buy, install and remove ship equipment. Equipment takes slots and power, and later affects combat, travel and trading. Version 1 builds the foundation and the one effect that can be used today: cargo capacity.

## Status
Not started. No equipment code exists in `src/`.

## Version 1 Scope
- [ ] Equipment definitions (data below) with category, quality, price and power
- [ ] Ship power and slot capacity per category (Cobra Mk III only; see Open Questions)
- [ ] Commander starts with the Basic item in each category (price 0, free), so a fresh ship matches today's behaviour
- [ ] `equipment` lists items for sale at the current system
- [ ] `equipment buy <item>` buys and installs; fails with a clear message if credits, slots or power are short
- [ ] `equipment sell <item>` removes an installed item and refunds 50% of its price
- [ ] `equipment installed` shows installed items, slots used and power used/available
- [ ] Cargo racks change cargo capacity (Standard 1.0x, Expanded 1.25x, Military 1.5x, Elite 2.0x of the ship's base capacity)
- [ ] Equipment is saved and loaded; old saves load with default equipment
- [ ] Commands listed in `help`

Not in v1: weapon, shield, engine, sensor and special effects (stored but inactive until combat and travel use them), tech-level and government restrictions, damage and repair, upgrades.

## Equipment Data

### Weapons
| Equipment | Quality | Price | Power | Damage | Accuracy | Range |
|-----------|---------|-------|-------|--------|----------|-------|
| Pulse Laser | Basic | 500 | 2 | 15 | 0.9 | 1000 |
| Beam Laser | Standard | 1200 | 4 | 25 | 0.8 | 1200 |
| Plasma Cannon | Military | 2500 | 6 | 45 | 0.6 | 800 |
| Missile Launcher | Elite | 5000 | 8 | 80 | 0.4 | 1500 |

### Shields
| Equipment | Quality | Price | Power | Protection | Recharge |
|-----------|---------|-------|-------|------------|----------|
| Basic Shield | Basic | 300 | 3 | 20 | 5 |
| Standard Shield | Standard | 800 | 5 | 40 | 8 |
| Military Shield | Military | 1800 | 7 | 70 | 12 |
| Elite Shield | Elite | 3500 | 10 | 100 | 15 |

### Engines
| Equipment | Quality | Price | Power | Speed | Fuel use |
|-----------|---------|-------|-------|-------|----------|
| Standard Drive | Basic | 200 | 2 | 1.0x | 1.0x |
| Performance Drive | Standard | 600 | 3 | 1.2x | 1.1x |
| Military Drive | Military | 1500 | 5 | 1.4x | 1.2x |
| Elite Drive | Elite | 3000 | 8 | 1.6x | 1.3x |

### Sensors
| Equipment | Quality | Price | Power | Range | Targeting |
|-----------|---------|-------|-------|-------|-----------|
| Basic Scanner | Basic | 150 | 1 | 1.0x | +0 |
| Advanced Scanner | Standard | 400 | 2 | 1.5x | +10 |
| Military Scanner | Military | 1000 | 4 | 2.0x | +25 |
| Elite Scanner | Elite | 2000 | 6 | 3.0x | +50 |

### Cargo Racks
| Equipment | Quality | Price | Power | Capacity |
|-----------|---------|-------|-------|----------|
| Standard Racks | Basic | 100 | 0 | 1.0x |
| Expanded Racks | Standard | 300 | 1 | 1.25x |
| Military Racks | Military | 800 | 2 | 1.5x |
| Elite Racks | Elite | 1500 | 3 | 2.0x |

### Special
| Equipment | Quality | Price | Power | Effect |
|-----------|---------|-------|-------|--------|
| Heat Sink | Basic | 200 | 1 | −30% heat |
| Chaff Launcher | Standard | 500 | 2 | +40% evasion |
| ECM System | Military | 1200 | 4 | +60% jamming |
| Cloaking Device | Elite | 2500 | 8 | Temporary invisibility |

## Ship Capacity (Cobra Mk III)
- Power: 20 units
- Slots: weapons 2, shields 1, engines 1, sensors 1, cargo 1, special 2

Cargo is one slot: racks replace each other, they don't stack. The earlier specs listed two cargo slots; one avoids stacking multipliers.

## Rules
- An item needs a free slot in its category and enough free power; otherwise the purchase fails and no credits are spent.
- Installed power cannot exceed the ship's capacity.
- Buying a second cargo rack replaces the current one (the old one is refunded at 50%).
- Cargo capacity cannot drop below the tons currently held: swapping to a smaller rack is refused if the hold would be over capacity.

## Later Phases
- **Effects:** weapons and shields in combat (08), engines and sensors in travel, special items in combat.
- **Availability:** tech level (Military TL8+, Elite TL9+), economy (Elite only in rich economies) and government (weapons banned in communist systems) restrict what each system sells.
- **Condition:** damage during events, repair at stations.
- **Upgrades, quality and rarity, crafting:** descoped.

## Open Questions
- **Prices vs income.** Equipment costs 100–5,000cr, but the player starts with 100cr and earns roughly 100cr per good trip. Either prices drop, income rises, or v1 accepts a long grind. Needs a decision before balancing.
- **Other ships.** Only the Cobra Mk III has power and slot numbers. Define them for the other ships in `data/ships.rs`, or derive them from the existing ship stats.
- **Ship swap.** What happens to installed equipment when the player buys a different ship (transfer, sell, or lose)?
- **Item ids.** Use the lowercase name with dashes (e.g. `pulse-laser`) as the `<item>` argument.

## Acceptance Criteria
- [ ] Buying installs the item and deducts credits
- [ ] Purchases fail without spending credits when slots, power or credits are short
- [ ] Selling refunds 50% and frees the slot and power
- [ ] Cargo capacity follows the installed racks and never drops below cargo held
- [ ] Equipment persists through save and load; old saves load with default equipment
- [ ] A new game plays exactly as it did before this feature

## Testing (write first)
- [ ] Install within slot and power limits; refuse over either limit
- [ ] Failed purchase leaves credits and equipment unchanged
- [ ] Sell refunds 50% and frees capacity
- [ ] Cargo rack changes capacity; downgrade refused when cargo would not fit
- [ ] Second cargo rack replaces the first
- [ ] Save/load round trip keeps equipment; old save loads with defaults
- [ ] `equipment installed` shows slots and power used

## Dependencies
- Feature 01: Core Gameplay ✅
- Feature 03: Trading ✅ (credits, cargo)
- Feature 04: Ship Upgrades ✅ (ship types)
- Feature 05: Save/Load ✅
