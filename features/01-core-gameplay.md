# Feature 01: Core Gameplay

## Overview
Implement the basic game loop and core mechanics that form the foundation of the Elite terminal trading game.

## Requirements

### Starting Conditions
- [x] Commander creation with customizable name
- [x] Starting location: Lave system
- [x] Starting credits: 100
- [x] Starting ship: Cobra Mk III with 20 tons cargo capacity
- [x] Empty cargo hold

### Basic Commands
- [x] `help` - Show available commands
- [x] `status` - Display ship and commander status
- [x] `quit` - Exit game

### Status Display
```
Commander: [Name]
Credits: [Amount]
Location: [System Name]
Ship: [Ship Type]
Cargo: [Used]/[Total] tons
Fuel: [Current]/[Max] light years
Hull: [Percentage]%
```

## Implementation Tasks

### 1. Project Structure Setup
- [x] Create main entry point
- [x] Create game/ directory structure
- [x] Create data/ directory for game data
- [x] Create utils/ directory for utilities

### 2. Core Game State
- [x] Implement commander.rs with player data structure
- [x] Implement ship.rs with basic ship properties
- [x] Create game state management system
- [x] Implement basic command parsing

### 3. Display System
- [x] Create utils/display.rs for terminal output
- [x] Implement status display formatting
- [x] Implement help command display
- [x] Add color support for terminal output

### 4. Input Handling
- [x] Create utils/input.rs for command processing
- [x] Implement command parsing and validation
- [x] Add error handling for invalid commands
- [x] Create command history (optional)

## Acceptance Criteria
- [x] Game starts and displays welcome message
- [x] Commander can be created with custom name
- [x] Status command shows all required information
- [x] Help command displays available commands
- [x] Quit command exits cleanly
- [x] All commands handle invalid input gracefully

## Testing
- [x] Test commander creation with various names
- [x] Test status display accuracy
- [x] Test help command completeness
- [x] Test quit functionality
- [x] Test invalid command handling

## Dependencies
- None (this is the foundation feature)
