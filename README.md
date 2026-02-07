# Elite Rust

A terminal-based Elite-style trading game written in Rust, featuring space exploration, commodity trading, and dynamic events.

## Features

Feature specs and implementation details live in the [features/](features/) directory.

### ✅ Implemented
- **01 – Core Gameplay**: Commander creation, `help` / `status` / `quit`, starting at Lave in a Cobra Mk III.
- **02 – Star Systems & Travel**: `jump`, `systems`, `info`; galaxy data and travel between systems.
- **03 – Trading**: `market`, `buy`, `sell`, `cargo`; commodities and per-system markets.
- **04 – Ship Upgrades**: `upgrade`, `ships`; buy/sell ships and upgrades.
- **05 – Save/Load**: `save`, `load`, `saves`, `delete`; multiple save slots, auto-save on quit.
- **06 – Dynamic Events**: Two-phase events (hyperspace + approach); government/economy modifiers; integrated into travel.

### 🚧 In Progress
- **07 – Weapons & Equipment**: Spec in place; equipment categories and mechanics (combat not yet wired).
- **09 – Equipment System**: Shop, slots, and effects; user commands and integration to complete.

### 📋 Planned
- **08 – Combat System**: Turn-based combat, enemy types, equipment integration (spec in features/).
- **10 – Reputation System**: Factions, effects, criminal consequences.
- **11 – Trading Log**: Transaction history, analytics, route performance.
- **13 – UI Improvements**: Color coding, table alignment, status indicators, menus.

## Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- Terminal with color support (recommended)

### Installation & Running

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd elitey_rust
   ```

2. **Build the project**
   ```bash
   make build
   # or
   cargo build
   ```

3. **Run the game**
   ```bash
   make run
   # or
   cargo run
   ```

4. **Run tests**
   ```bash
   make test
   # or
   cargo test
   ```

## Available Commands

### Core
- `help` - Show available commands
- `status` - Display ship and commander status
- `quit` / `exit` - Exit the game (auto-saves)

### Trading
- `market` - Show local market prices
- `buy <commodity> [amount]` - Buy cargo
- `sell <commodity> [amount]` - Sell cargo
- `cargo` - List cargo hold

### Travel
- `jump <system name>` - Hyperspace to another system (triggers travel events)
- `systems` - List systems in range
- `info <system name>` - Info on a system

### Ships & Upgrades
- `upgrade` - Show upgrade options
- `upgrade buy <item>` - Buy an upgrade
- `ships` - List ships for sale

### Save/Load
- `save [name]` - Save game
- `load [name]` - Load game
- `saves` - List save files
- `delete <name>` - Delete a save

### Game Information
The status command displays:
```
Commander: [Name]
Credits: [Amount]
Location: [System Name]
Ship: [Ship Type]
Cargo: [Used]/[Total] tons
Fuel: [Current]/[Max] light years
Hull: [Percentage]%
```

## Development

### Project Structure
```
src/
├── main.rs              # Entry point and game loop
├── game/                # Core game logic
│   ├── mod.rs
│   ├── commander.rs     # Player data and operations
│   ├── ship.rs          # Ship management and stats
│   ├── game_state.rs    # Overall game state
│   ├── trading.rs       # Market and cargo
│   ├── travel.rs        # Jump and system navigation
│   ├── events.rs        # Dynamic travel events (two-phase)
│   ├── upgrades.rs      # Ship and equipment upgrades
│   └── save_load.rs     # Save/load game state
├── data/                # Static and persisted data
│   ├── systems.rs       # Star systems and galaxy
│   ├── commodities.rs   # Commodity definitions
│   ├── ships.rs         # Ship types and stats
│   └── save_load.rs     # Save file I/O
└── utils/
    ├── mod.rs
    ├── display.rs       # Terminal output and formatting
    └── input.rs         # Input processing and validation
features/                # Feature specs (see AGENTS.md)
tests/                   # Integration / BDD-style tests
```

### Makefile Targets
- `make build` - Build the project
- `make test` - Run all tests
- `make run` - Run the game
- `make lint` - Run linter (clippy)
- `make check` - Check if code compiles
- `make format` - Format code with rustfmt
- `make clean` - Clean build artifacts
- `make all-checks` - Run lint, test, and check
- `make help` - Show available targets

### Testing
- **Unit tests**: In-module `#[cfg(test)]` tests where present.
- **Integration / BDD**: Feature-first specs in `features/`; tests added per feature.

Run tests with `make test` or `cargo test`.

### Code Quality
- **Linting**: Uses `clippy` for code quality checks
- **Formatting**: Uses `rustfmt` for consistent code style
- **Error Handling**: Comprehensive error handling with `anyhow`
- **Documentation**: Inline documentation for all public APIs

## Game Design

### Starting Conditions
- **Commander**: Custom name
- **Credits**: 100 (starting capital)
- **Location**: Lave system
- **Ship**: Cobra Mk III
- **Cargo**: 20 tons capacity, empty hold
- **Fuel**: 7 light years capacity, full tank
- **Hull**: 100% integrity

### Core Mechanics
- **Command Processing**: Robust input parsing and validation
- **State Management**: Immutable game state with proper error handling
- **Display System**: Color-coded terminal output with status indicators
- **Error Handling**: Graceful handling of invalid commands and edge cases

## Contributing

1. Follow the existing code style and patterns
2. Write tests for new features
3. Update documentation as needed
4. Use the Makefile targets for development workflow

## License

[Add your license information here]

## Acknowledgments

Inspired by the classic Elite series of space trading games.
