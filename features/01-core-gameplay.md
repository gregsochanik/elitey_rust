# Feature 01: Core Gameplay

## Overview
Implement the basic game loop and core mechanics that form the foundation of the Elite terminal trading game.

## Requirements

### Starting Conditions
- [ ] Commander creation with customizable name
- [ ] Starting location: Lave system
- [ ] Starting credits: 100
- [ ] Starting ship: Cobra Mk III with 20 tons cargo capacity
- [ ] Empty cargo hold

### Basic Commands
- [ ] `help` - Show available commands
- [ ] `status` - Display ship and commander status
- [ ] `quit` - Exit game

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
- [ ] Create main entry point
- [ ] Create game/ directory structure
- [ ] Create data/ directory for game data
- [ ] Create utils/ directory for utilities

### 2. Core Game State
- [ ] Implement commander.rs with player data structure
- [ ] Implement ship.rs with basic ship properties
- [ ] Create game state management system
- [ ] Implement basic command parsing

### 3. Display System
- [ ] Create utils/display.rs for terminal output
- [ ] Implement status display formatting
- [ ] Implement help command display
- [ ] Add color support for terminal output

### 4. Input Handling
- [ ] Create utils/input.rs for command processing
- [ ] Implement command parsing and validation
- [ ] Add error handling for invalid commands
- [ ] Create command history (optional)

## Acceptance Criteria
- [ ] Game starts and displays welcome message
- [ ] Commander can be created with custom name
- [ ] Status command shows all required information
- [ ] Help command displays available commands
- [ ] Quit command exits cleanly
- [ ] All commands handle invalid input gracefully

## Testing
- [ ] Test commander creation with various names
- [ ] Test status display accuracy
- [ ] Test help command completeness
- [ ] Test quit functionality
- [ ] Test invalid command handling

## Dependencies
- None (this is the foundation feature)
