# Feature 13: UI Improvements - High Impact, Low Effort

## Overview
Enhance the visual appeal and usability of the CLI interface with minimal code changes while maintaining the existing functionality.

## Requirements

- [ ] Favour any out of the box rust CLI improvement modules for this rather than low level coding solutions 

### Color Coding System
- [ ] Credits/Money: Green for positive amounts, red for negative
- [ ] System Names: Blue for current system, cyan for destinations
- [ ] Equipment Categories: Different colors per type (weapons=red, shields=blue, etc.)
- [ ] Status Indicators: Green for good, yellow for warnings, red for critical
- [ ] Error Messages: Red text with clear formatting
- [ ] Success Messages: Green text for successful actions

### Enhanced Table Alignment
- [ ] Consistent column widths across all tables
- [ ] Right-aligned numbers (prices, quantities, stats)
- [ ] Left-aligned text (names, descriptions)
- [ ] Proper spacing between columns
- [ ] Clear header separation from data

### Status Indicators
- [ ] Fuel Gauge: `[████████░░] 80%` format with color coding
- [ ] Cargo Capacity: `[██████░░░░] 60/100 tons` format
- [ ] Hull Status: Visual hull integrity bar
- [ ] Power Usage: `[████░░░░░░] 40/100 MW` format

### Numbered Menu Options
- [ ] Sequential numbering for all menu options
- [ ] `1. Status` format with proper spacing
- [ ] Accept both numbers and text for commands
- [ ] Show numbered options in help text
- [ ] Context-aware renumbering

## Implementation Tasks

### 1. Color System Foundation
- [ ] Implement color detection for terminal compatibility
- [ ] Add fallback for terminals without color support
- [ ] Create colorize() function for text formatting

### 2. Status Bar System
- [ ] Create status bar generation functions
- [ ] Implement percentage calculation and bar drawing
- [ ] Add color coding based on percentage levels
- [ ] Integrate status bars into existing displays

### 3. Table Formatting
- [ ] Update table formatting functions
- [ ] Implement consistent column alignment
- [ ] Add proper spacing and padding
- [ ] Update market, cargo, and equipment displays
- [ ] Create reusable table formatting system
- [ ] Add column separators and proper borders
- [ ] Create unified table rendering system
- [ ] Ensure consistent borders and formatting across all tables

### 4. Menu System
- [ ] Add numbered menu handling
- [ ] Update main menu display 
- [ ] Modify command processing for number input
- [ ] Update help text to show numbered options

## User Experience Examples

### Status Display
```
Commander: Greg
Credits: 1500 (green)
Location: Leesti (blue)
Ship: Cobra Mk III
Cargo: [██████░░░░] 60/100 tons
Fuel: [████████░░] 80%
Hull: [██████████] 100%
```

### Market Display
```
Market at Leesti
================
Food:      Buy: 16cr  Sell: 14cr  (green prices)
Machinery: Buy: 49cr  Sell: 46cr  (green prices)
```

### Menu Options
```
1. Status
2. Market
3. Cargo
4. Equipment
5. Travel
6. Save Game
7. Quit

Enter choice (1-7): 
```

## Acceptance Criteria
- [ ] All monetary values are color-coded appropriately
- [ ] System names are visually distinct with colors
- [ ] Equipment categories have consistent color coding
- [ ] Status bars display for fuel, cargo, hull, and power
- [ ] All tables have consistent alignment and spacing
- [ ] All menus have numbered options
- [ ] Color support gracefully degrades for unsupported terminals
- [ ] No existing functionality is broken
- [ ] Performance impact is minimal

## Testing
- [ ] Test color display on different terminal types
- [ ] Verify status bar calculations are accurate
- [ ] Test table alignment with various data lengths
- [ ] Verify numbered menu navigation works correctly
- [ ] Test fallback behavior for unsupported terminals
