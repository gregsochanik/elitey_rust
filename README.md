# Elite Rust

A terminal-based Elite-style trading game written in Rust, featuring space exploration, commodity trading, and dynamic events.

## Features

### ✅ Feature 01: Core Gameplay (Implemented)
- **Commander Creation**: Customizable commander name with starting conditions
- **Starting Conditions**: 100 credits, Lave system, Cobra Mk III ship with 20 tons cargo
- **Basic Commands**: `help`, `status`, `quit`
- **Status Display**: Complete ship and commander information
- **Command Processing**: Robust input handling with error management

### 🚧 Planned Features
- **Feature 02**: Star Systems & Travel
- **Feature 03**: Trading System
- **Feature 04**: Ship Upgrades
- **Feature 05**: Save/Load System
- **Feature 06**: Dynamic Events
- **Feature 07**: Weapons & Equipment
- **Feature 08**: Combat System
- **Feature 09**: Equipment System
- **Feature 10**: Reputation System
- **Feature 11**: Trading Log
- **Feature 12**: Quality of Life Features
- **Feature 13**: UI Improvements

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

### Core Commands
- `help` - Show available commands
- `status` - Display ship and commander status
- `quit` - Exit the game

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
│   └── game_state.rs    # Overall game state
└── utils/               # Utility functions
    ├── mod.rs
    ├── display.rs       # Terminal output and formatting
    └── input.rs         # Input processing and validation
tests/
└── integration_tests.rs # Integration tests
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
The project includes comprehensive tests:
- **Unit Tests**: Individual component testing
- **Integration Tests**: End-to-end functionality testing
- **BDD Style**: Feature-first testing approach

Run tests with:
```bash
make test
```

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
