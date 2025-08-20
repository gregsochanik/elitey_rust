# Feature 06: Dynamic Events While Jumping

## 🎯 **Objective**
Add surprise and tension between system jumps with random events that can reward, challenge, or endanger the player.

## 📋 **Requirements**

### **Core Events**
- [ ] **Alien Encounter** 👽 — attempt to communicate, flee, or fight
- [ ] **Pirate Attack** 🏴‍☠️ — risk cargo loss or damage  
- [ ] **Rescue Signal** 🛸 — optional diversion for a reward (or trap)
- [ ] **Wormhole** 🌌 — unpredictable jump to random coordinates

### **Event Mechanics**
- [ ] Events trigger randomly during `jump` command (10-15% chance)
- [ ] Each event has multiple outcome choices
- [ ] Events can affect: credits, cargo, ship damage, reputation
- [ ] Events can lead to new locations or system changes

### **Event Types & Outcomes**

#### **👽 Alien Encounter**
- **Communicate**: +reputation, potential trade opportunity
- **Flee**: -fuel, possible ship damage
- **Fight**: -hull damage, potential cargo loss, +credits if successful

#### **🏴‍☠️ Pirate Attack**
- **Fight**: -hull damage, potential cargo loss, +credits if successful
- [ ] **Flee**: -fuel, possible cargo loss
- **Surrender**: -cargo, -credits, but no damage

#### **🛸 Rescue Signal**
- **Investigate**: 50% chance of reward, 50% chance of trap
- **Ignore**: No effect
- **Reward**: +credits, +reputation, rare cargo
- **Trap**: -hull damage, -cargo, pirate ambush

#### **🌌 Wormhole**
- **Enter**: Random jump to unknown system, +fuel cost
- **Avoid**: Continue to original destination
- **Effects**: Can discover new systems, get lost, find rare commodities

## 🛠 **Implementation Tasks**

### **Phase 1: Event System Foundation**
- [ ] Create `game/events.rs` module
- [ ] Implement event triggering system (random chance during jumps)
- [ ] Create base event structure with choices and outcomes
- [ ] Add event logging and display functions

### **Phase 2: Core Events**
- [ ] Implement **Alien Encounter** event
- [ ] Implement **Pirate Attack** event  
- [ ] Implement **Rescue Signal** event
- [ ] Implement **Wormhole** event

### **Phase 3: Integration & Polish**
- [ ] Integrate events into `game/travel.rs` jump system
- [ ] Add event outcomes to commander status (reputation, damage)
- [ ] Create event-specific UI messages and choices
- [ ] Add event history tracking

### **Phase 4: Advanced Features**
- [ ] Add event difficulty scaling with player progress
- [ ] Implement event-specific rewards (rare commodities, upgrades)
- [ ] Add reputation system affecting event outcomes
- [ ] Create event chains (consequences of previous choices)

## ✅ **Acceptance Criteria**

### **Functional Requirements**
- [ ] Events trigger randomly during system jumps
- [ ] Each event presents clear choices to the player
- [ ] Events have meaningful consequences (credits, cargo, damage)
- [ ] Event outcomes are properly logged and displayed
- [ ] Events can lead to new locations or system discoveries

### **User Experience**
- [ ] Event messages are clear and engaging
- [ ] Player choices are intuitive and well-explained
- [ ] Event outcomes are immediately visible
- [ ] Events add excitement without being overwhelming

### **Technical Requirements**
- [ ] Events integrate seamlessly with existing travel system
- [ ] Event system is modular and easily extensible
- [ ] Event outcomes properly update game state
- [ ] Random generation is fair and balanced

## 🧪 **Testing**

### **Unit Tests**
- [ ] Test event triggering probability
- [ ] Test each event type and its outcomes
- [ ] Test event integration with travel system
- [ ] Test event state management

### **Integration Tests**
- [ ] Test events with different ship types and cargo loads
- [ ] Test events at different player progression levels
- [ ] Test event chains and consequences
- [ ] Test event balance and fairness

### **Manual Testing**
- [ ] Verify event triggering during normal gameplay
- [ ] Test all event choices and outcomes
- [ ] Verify event consequences update game state correctly
- [ ] Test event replayability and variety

## 📊 **Estimated Effort**
- **Phase 1**: 2-3 hours ✅
- **Phase 2**: 4-5 hours ✅  
- **Phase 3**: 2-3 hours ✅
- **Phase 4**: 3-4 hours
- **Total**: 11-15 hours (Core system completed)

## 🔗 **Dependencies**
- Feature 02: Star Systems & Travel (for jump system integration)
- Feature 03: Trading System (for cargo management)
- Feature 04: Ship Upgrades (for damage/repair mechanics)

## 🎮 **Gameplay Impact**
- **Excitement**: Adds unpredictability and tension to travel
- **Strategy**: Players must consider risks vs rewards
- **Replayability**: Different events create unique experiences
- **Progression**: Events can unlock new opportunities and challenges 

## **🎯 Current Dynamic Events System Status:**

### **✅ What's Already Implemented:**

#### **1. Core Event Types (All 4 from spec):**
- **👽 Alien Encounter** - communicate, flee, fight
- **🏴‍☠️ Pirate Attack** - fight, flee, surrender  
- **🛸 Rescue Signal** - investigate (50% trap), ignore
- **🌌 Wormhole** - enter (random jump), avoid

#### **2. Event Mechanics:**
- **12% trigger chance** during jumps (spec said 10-15%)
- **Multiple choices** for each event
- **Consequences:** credits, cargo loss, hull damage, fuel, reputation
- **Random jump** via wormholes

#### **3. Integration:**
- ✅ Integrated into `game/travel.rs` jump system
- ✅ Proper UI with choices and outcomes
- ✅ Event effects update game state

### **🚀 Enhancement Opportunities:**

Based on our earlier discussion about economy/government modifiers, here are some **great improvements** we could add:

#### **1. Economy-Based Event Modifiers:**
-- Rich economies: Lower pirate risk, higher rescue rewards
-- Poor economies: Higher pirate risk, more desperate rescue signals
-- Industrial systems: More machinery-related events
-- Agricultural systems: Food shortage events


#### **2. Government-Based Event Modifiers:**
-- Communist: Government inspection events, restricted trade
-- Corporate: Business opportunity events, luxury trade
-- Feudal: Noble house events, ancient artifacts
-- Dictatorship: Military patrols, smuggling opportunities


#### **3. New Event Types:**
- **🛡️ Government Inspection** (Communist/Dictatorship)
- **💼 Business Opportunity** (Corporate/Rich economies)
- **🏰 Noble House Event** (Feudal systems)
- **🚨 Military Patrol** (Dictatorship systems)
- **🌾 Food Shortage** (Agricultural systems)
- **⚙️ Industrial Accident** (Industrial systems)

#### **4. Enhanced Existing Events:**
- **Pirate Attacks:** Different types based on economy (desperate poor pirates vs sophisticated rich pirates)
- **Rescue Signals:** More specific based on system type
- **Alien Encounters:** Different alien types based on tech level

Would you like me to implement some of these enhancements? I could start with:

1. **Economy-based event probability modifiers**
2. **Government-specific new event types**
3. **Enhanced existing events with system-specific variations**

Which direction interests you most? 🚀

## **🔄 Two-Phase Travel Event System**

### **🎯 Objective**
Split travel events into two distinct phases: **Hyperspace** and **Approach**, with separate event types and probability rolls for each phase.

### **📋 Requirements**

#### **Phase 1: Hyperspace Events**
- **🌌 Wormhole** - Random jump to unknown system
- **⚡ Hyperspace Malfunction** - System damage, fuel loss
- **🌀 Space Anomaly** - Unpredictable effects, rare rewards
- **Event Probability**: 8-12% chance during hyperspace jump
- **Timing**: Occurs during the jump sequence itself

#### **Phase 2: Approach Events**
- **🏴‍☠️ Pirate Attack** - Combat encounter, cargo risk
- **👽 Alien Encounter** - Communication, trade, or combat
- **🛸 Rescue Signal** - Optional diversion, reward or trap
- **🛡️ Government Inspection** - Authority encounter, reputation effects
- **🚨 Military Patrol** - Military encounter, different outcomes
- **Event Probability**: 10-15% chance during approach to station
- **Timing**: Occurs after hyperspace but before docking

### **🛠 Implementation Tasks**

#### **Phase 1: Event System Restructure**
- [ ] Create `process_hyperspace_event()` function
- [ ] Create `process_approach_event()` function
- [ ] Modify `process_travel_event()` to handle both phases
- [ ] Add separate probability calculations for each phase

#### **Phase 2: Event Type Categorization**
- [ ] Categorize existing events into hyperspace vs approach
- [ ] Add new hyperspace-specific events
- [ ] Add new approach-specific events
- [ ] Update event selection logic

#### **Phase 3: Enhanced Travel Narrative**
- [ ] Update travel display to show both phases
- [ ] Add clear phase separation in narrative
- [ ] Display events with phase context
- [ ] Show separate probability explanations

#### **Phase 4: Testing & Balance**
- [ ] Add comprehensive tests for both phases
- [ ] Test independent probability rolls
- [ ] Test events occurring in both phases
- [ ] Balance event probabilities and outcomes

### **✅ Acceptance Criteria**

#### **Functional Requirements**
- [ ] Events trigger in two separate phases during travel
- [ ] Each phase has independent probability rolls
- [ ] Players can experience events in both phases, neither, or just one
- [ ] Event types are appropriate for their phase
- [ ] Travel narrative clearly shows phase separation

#### **User Experience**
- [ ] Clear distinction between hyperspace and approach events
- [ ] Intuitive event choices for each phase
- [ ] Meaningful consequences that fit the phase context
- [ ] Engaging narrative that builds tension

#### **Technical Requirements**
- [ ] Modular event system that supports phase separation
- [ ] Independent probability calculations for each phase
- [ ] Proper integration with existing travel system
- [ ] Comprehensive test coverage for both phases

### **🎮 Gameplay Impact**
- **Enhanced Immersion**: Realistic two-phase travel experience
- **Strategic Depth**: Different risks and rewards for each phase
- **Increased Variety**: More event combinations and outcomes
- **Better Pacing**: Natural break between jump and approach phases 