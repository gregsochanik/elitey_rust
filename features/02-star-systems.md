# Feature 02: Star Systems & Travel

## Overview
Implement the star system network, travel mechanics, and basic navigation between systems.

## Requirements

### Star System Data
- [ ] System name and unique identifier
- [ ] Economy type (Agricultural, Industrial, High-Tech, etc.)
- [ ] Distance from Lave in light years
- [ ] Available commodities list
- [ ] System description/flavor text

### Travel Mechanics
- [ ] `jump <system>` command to travel between systems
- [ ] Fuel consumption based on distance
- [ ] Travel time calculation
- [ ] Cannot travel beyond fuel range
- [ ] Fuel refueling mechanics

### System Information
- [ ] `systems` command to list available destinations
- [ ] `info <system>` command to show system details
- [ ] Display distance and fuel requirements
- [ ] Show economy type and available commodities

## Implementation Tasks

### 1. System Data Structure
- [ ] Create data/systems.rs with system definitions
- [ ] Define economy types and their characteristics
- [ ] Create system lookup and validation functions
- [ ] Implement distance calculation between systems

### 2. Travel System
- [ ] Implement fuel consumption logic
- [ ] Create travel validation (fuel check, distance check)
- [ ] Add travel time simulation
- [ ] Implement fuel refueling at stations

### 3. Navigation Commands
- [ ] Implement `jump` command with validation
- [ ] Implement `systems` command to list destinations
- [ ] Implement `info` command for system details
- [ ] Add fuel status to status display

### 4. System Display
- [ ] Create system information display formatting
- [ ] Show available systems with distance and fuel cost
- [ ] Display system economy and commodity availability
- [ ] Add travel confirmation prompts

## Acceptance Criteria
- [ ] Can travel between connected systems
- [ ] Fuel consumption is accurate to distance
- [ ] Cannot travel without sufficient fuel
- [ ] System information is displayed correctly
- [ ] Travel commands handle invalid destinations gracefully
- [ ] Fuel status is updated after travel

## Testing
- [ ] Test travel between all connected systems
- [ ] Test fuel consumption accuracy
- [ ] Test travel with insufficient fuel
- [ ] Test system information display
- [ ] Test invalid system names

## Dependencies
- Feature 01: Core Gameplay (commander, ship, basic commands)
