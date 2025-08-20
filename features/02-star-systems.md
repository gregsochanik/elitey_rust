# Feature 02: Star Systems & Travel

## Overview
Implement the star system network, travel mechanics, and basic navigation between systems.

## Requirements

### Star System Data
- [x] System name and unique identifier
- [x] Economy type (Agricultural, Industrial, High-Tech, etc.)
- [x] Distance from Lave in light years
- [x] Available commodities list
- [x] System description/flavor text
- [x] Load star system data from galaxies/galaxy1.json file
- [x] Use actual 3D coordinates (x, y) for distance calculations
- [x] Include government types and tech levels from galaxy data
- [x] Use the original Elite distance calculation:
```c
/* Calculate distance between two planets (scaled to light years) */
int distance(int x1, int y1, int x2, int y2)
{
    int dx = x1 - x2;
    int dy = y1 - y2;
    return (int)(sqrt((dx * dx) + (dy * dy)) / 4);
}
```

### Travel Mechanics
- [x] `jump <system>` command to travel between systems
- [x] Fuel consumption based on distance
- [x] Travel time calculation
- [x] Cannot travel beyond fuel range
- [x] Fuel refueling mechanics

### System Information
- [x] `systems` command to list available destinations
- [x] `info <system>` command to show system details
- [x] Display distance and fuel requirements
- [x] Show economy type and available commodities

## Implementation Tasks

### 1. System Data Structure
- [x] Create data/systems.rs with system definitions
- [x] Define economy types and their characteristics
- [x] Create system lookup and validation functions
- [x] Implement distance calculation between systems
- [ ] Parse and load galaxy data from JSON file
- [ ] Implement 3D coordinate-based distance calculations
- [ ] Add government and tech level data structures

### 2. Travel System
- [x] Implement fuel consumption logic
- [x] Create travel validation (fuel check, distance check)
- [x] Add travel time simulation
- [x] Implement fuel refueling at stations

### 3. Navigation Commands
- [x] Implement `jump` command with validation
- [x] Implement `systems` command to list destinations
- [x] Implement `info` command for system details
- [x] Add fuel status to status display

### 4. System Display
- [x] Create system information display formatting
- [x] Show available systems with distance and fuel cost
- [x] Display system economy and commodity availability
- [x] Add travel confirmation prompts

## Acceptance Criteria
- [x] Can travel between connected systems
- [x] Fuel consumption is accurate to distance
- [x] Cannot travel without sufficient fuel
- [x] System information is displayed correctly
- [x] Travel commands handle invalid destinations gracefully
- [x] Fuel status is updated after travel
- [ ] All star systems from galaxy1.json are available for travel
- [ ] Distance calculations use actual 3D coordinates
- [ ] System information includes government and tech level data

## Testing
- [x] Test travel between all connected systems
- [x] Test fuel consumption accuracy
- [x] Test travel with insufficient fuel
- [x] Test system information display
- [x] Test invalid system names
- [ ] Test loading of all galaxy systems from JSON
- [ ] Test 3D coordinate distance calculations
- [ ] Test government and tech level data display

## Dependencies
- Feature 01: Core Gameplay (commander, ship, basic commands)
