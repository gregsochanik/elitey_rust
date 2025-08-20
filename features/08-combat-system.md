# Feature 08: Turn-Based Combat System

## Overview
Implement a turn-based combat system for hostile encounters during space travel. Different enemy types will have unique strengths, weaknesses, and combat behaviors, requiring strategic thinking from the player.

## Combat System Design Decision
**Hybrid System**: Combining D&D 5e mechanics with Elite Dangerous space combat and XCOM strategic elements.

### Core System Foundation
- **D&D 5e Base**: Initiative, attack rolls, damage calculation, action economy
- **Elite Dangerous Space Combat**: Shield/hull separation, power management, heat system
- **XCOM Strategic Elements**: Positioning, cover, reactions, tactical movement

## Requirements

### Core Combat Mechanics
- [ ] Turn-based combat system with player and enemy turns
- [ ] Combat state management (in-combat, victory, defeat)
- [ ] Combat UI showing health, shields, and combat options
- [ ] Combat log showing actions and results
- [ ] Escape mechanism with success/failure chance

### Combat System Mechanics

#### Initiative System (D&D-inspired)
- [ ] **Initiative Roll**: 1d20 + Speed modifier for turn order
- [ ] **Turn Order**: Player typically goes first (hero advantage)
- [ ] **Speed Modifiers**: Equipment and ship type affect initiative

#### Attack System (D&D-inspired)
- [ ] **Attack Roll**: 1d20 + Accuracy vs Enemy Dodge
- [ ] **Critical Hits**: Natural 20 = critical, special effects
- [ ] **Advantage/Disadvantage**: Situational modifiers
- [ ] **Damage Calculation**: Weapon Damage + Equipment + Critical

#### Space Combat Elements (Elite-inspired)
- [ ] **Shield/Hull Separation**: Different health pools
- [ ] **Shield Regeneration**: Automatic per turn
- [ ] **Power Management**: Shield/Weapon/Engine allocation
- [ ] **Heat System**: Actions generate heat, affects performance
- [ ] **Weapon Types**: Kinetic, Thermal, Explosive damage types

#### Strategic Elements (XCOM-inspired) DESCOPED
- [ ] **Positioning**: Distance affects accuracy and damage
- [ ] **Cover System**: Defensive bonuses based on position
- [ ] **Reactions**: Overwatch and counter-attack opportunities
- [ ] **Tactical Movement**: Position changes as separate action

### Action Economy (D&D-inspired) DESCOPED
- [ ] **Standard Action**: Attack, Defend, Use Equipment
- [ ] **Move Action**: Change position (affects accuracy)
- [ ] **Bonus Action**: Quick actions (shield boost, emergency repair)
- [ ] **Reaction**: Overwatch, counter-attacks, defensive responses

### Combat Statistics

#### Player Ship Stats
- [ ] **Hull Health**: Ship structural integrity (0-100)
- [ ] **Shield Strength**: Energy shield protection (0-100, regenerates)
- [ ] **Weapon Damage**: Base attack power
- [ ] **Accuracy**: Hit chance modifier
- [ ] **Dodge**: Evasion chance
- [ ] **Speed**: Initiative and movement modifier
- [ ] **Heat**: Current heat level (0-100, affects performance)

#### Enemy Stats
- [ ] **Health**: Enemy ship integrity
- [ ] **Shields**: Enemy shield strength
- [ ] **Damage**: Enemy attack power
- [ ] **Accuracy**: Enemy hit chance
- [ ] **Dodge**: Enemy evasion
- [ ] **Speed**: Enemy initiative and movement
- [ ] **Special Abilities**: Unique combat powers

### Enemy Types and Characteristics

#### Pirate Raiders
- **Strength**: High damage, aggressive tactics
- **Weakness**: Poor accuracy, low shields
- **Behavior**: Direct attacks, minimal strategy
- **Combat Style**: Offensive, high-risk high-reward
- **Special**: Desperate attacks (high damage when low health)

#### Corporate Security
- **Strength**: Balanced stats, good shields
- **Weakness**: Predictable patterns, moderate damage
- **Behavior**: Defensive tactics, shield-focused
- **Combat Style**: Defensive, shield regeneration
- **Special**: Shield boost ability

#### Alien Scouts
- **Strength**: High accuracy, special abilities
- **Weakness**: Low health, vulnerable to certain weapons
- **Behavior**: Hit-and-run tactics, special attacks
- **Combat Style**: Tactical, uses special abilities
- **Special**: Cloaking device, precision strikes

#### Military Patrol
- **Strength**: High shields, good accuracy
- **Weakness**: Slow, predictable attack patterns
- **Behavior**: Systematic approach, coordinated attacks
- **Combat Style**: Methodical, shield-heavy
- **Special**: Coordinated attacks, tactical positioning

### Player Combat Options

#### Attack Actions
- [ ] **Standard Attack**: Basic weapon damage (1d20 + Accuracy)
- [ ] **Precise Shot**: Higher accuracy, lower damage (Advantage on attack)
- [ ] **Heavy Strike**: High damage, lower accuracy (Disadvantage on attack, +50% damage)
- [ ] **Target Systems**: Attack specific enemy systems (shields, weapons, engines)

#### Defensive Actions
- [ ] **Raise Shields**: Increase shield strength (Bonus action)
- [ ] **Evasive Maneuvers**: Increase dodge chance (Move action)
- [ ] **Repair Systems**: Heal hull damage (Standard action)
- [ ] **Full Defense**: Maximum defensive stance (Advantage on dodge)

#### Special Actions
- [ ] **Use Equipment**: Activate installed equipment (weapons, shields, etc.)
- [ ] **Emergency Repair**: Quick hull repair at cost (Bonus action, generates heat)
- [ ] **Tactical Retreat**: Attempt to escape combat (Standard action, roll vs enemy speed)
- [ ] **Surrender**: Give up (results in cargo loss but ship preservation)

### Combat Flow

#### Pre-Combat
- [ ] Enemy type identification
- [ ] Combat environment setup
- [ ] Initiative rolls for turn order
- [ ] Initial positioning and stats display

#### Combat Rounds
- [ ] **Initiative Phase**: Determine turn order
- [ ] **Player Turn**: Choose action (Attack/Defend/Special)
- [ ] **Action Resolution**: Roll dice, calculate damage
- [ ] **Enemy AI Decision**: AI chooses action based on type
- [ ] **Enemy Action**: Execute enemy action
- [ ] **Status Phase**: Process effects, heat, shield regeneration
- [ ] **Combat Log**: Update with results

#### Post-Combat
- [ ] Victory rewards (credits, cargo, reputation)
- [ ] Defeat consequences (cargo loss, hull damage, credits)
- [ ] Combat experience and skill progression
- [ ] Return to travel mode

### Equipment Integration

#### Combat Equipment
- [ ] **Weapons**: Laser cannons, missile launchers, plasma weapons
- [ ] **Shields**: Standard shields, reinforced shields, adaptive shields
- [ ] **Engines**: Standard, military, experimental
- [ ] **Special Systems**: ECM, cloaking devices, repair drones

#### Equipment Effects
- [ ] **Damage Modifiers**: Based on weapon type vs enemy type
- [ ] **Shield Strength**: Base shield capacity and regeneration rate
- [ ] **Accuracy Bonuses**: Equipment affects hit chance
- [ ] **Heat Management**: Equipment affects heat generation and dissipation
- [ ] **Special Abilities**: Unique equipment powers and cooldowns

### AI Combat Behavior

#### Difficulty Levels
- [ ] **Easy**: Predictable patterns, lower stats, basic AI
- [ ] **Normal**: Balanced challenge, varied tactics, moderate AI
- [ ] **Hard**: Aggressive AI, higher stats, special abilities, advanced tactics

#### AI Tactics by Enemy Type
- [ ] **Aggressive**: Focus on damage, ignore defense (Pirates)
- [ ] **Defensive**: Prioritize shields and survival (Corporate)
- [ ] **Tactical**: Balance offense and defense (Military)
- [ ] **Adaptive**: Change tactics based on player actions (Aliens)

### Combat Rewards and Consequences

#### Victory Rewards
- [ ] Credits from defeated enemies
- [ ] Salvaged equipment and cargo
- [ ] Reputation gains in nearby systems
- [ ] Combat experience points
- [ ] Special items or rare commodities

#### Defeat Consequences
- [ ] Cargo loss (partial or complete)
- [ ] Hull damage requiring repairs
- [ ] Credit penalties
- [ ] Reputation losses
- [ ] Forced retreat to nearest system

### Implementation Tasks

#### Phase 1: Core Combat System
- [ ] Create combat state management
- [ ] Implement D&D-style initiative and attack rolls
- [ ] Add basic damage calculation system
- [ ] Create simple enemy AI
- [ ] Implement shield/hull separation

#### Phase 2: Enemy Types and AI
- [ ] Design enemy type characteristics
- [ ] Implement different AI behaviors
- [ ] Add special abilities for enemies
- [ ] Create enemy-specific combat patterns

#### Phase 3: Equipment Integration
- [ ] Connect equipment to combat stats
- [ ] Implement equipment effects in combat
- [ ] Add equipment-based special actions
- [ ] Create equipment upgrade effects

#### Phase 4: Advanced Features (DESCOPED)
- [ ] Add heat management system (DESCOPED)
- [ ] Implement positioning and cover (DESCOPED)
- [ ] Create reaction system (overwatch) (DESCOPED)
- [ ] Add combat experience and progression (DESCOPED)

### Technical Requirements

#### New Modules
- [ ] `game/combat.rs` - Main combat system
- [ ] `game/enemies.rs` - Enemy definitions and AI
- [ ] `data/enemies.rs` - Enemy definitions
- [ ] `utils/dice.rs` - Dice rolling utilities

#### Integration Points
- [ ] Connect to existing event system
- [ ] Integrate with equipment system
- [ ] Update save/load for combat state
- [ ] Add combat statistics to commander data

### Testing Requirements
- [ ] Unit tests for combat mechanics
- [ ] Integration tests for enemy AI
- [ ] Balance testing for different enemy types
- [ ] Performance testing for combat calculations
- [ ] User experience testing for combat flow

## Success Criteria
- [ ] Engaging turn-based combat that feels strategic
- [ ] Distinct enemy types with unique behaviors
- [ ] Meaningful equipment choices affecting combat
- [ ] Balanced rewards and consequences
- [ ] Smooth integration with existing game systems
- [ ] Clear and intuitive combat interface

## Future Enhancements (DESCOPED)
- [ ] Multi-enemy combat scenarios (DESCOPED)
- [ ] Fleet battles with multiple ships (DESCOPED)
- [ ] Boarding actions and crew combat (DESCOPED)
- [ ] Alliance system for cooperative combat (DESCOPED)
- [ ] Tournament and arena combat modes (DESCOPED)