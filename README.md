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

- **07 – Equipment** (v1): `equipment`, `equipment buy|sell|installed`; slots, power and cargo racks. Other effects wait for combat and travel.
- **08 – Combat** (v1): choosing Fight against pirates starts a turn-based fight using your weapons and shield; win loot, lose cargo, flee or surrender.
- **16 – Jobs** (v1): `jobs`, `accept`, `missions` and `abandon`; courier jobs pay on arrival, bounty jobs start a pirate fight on arrival, and both fail when their jumps run out.
- **14 – Refuelling** (v1): `refuel [units]` tops up the tank for credits.
- **12 – Ship Repair** (v1): `repair [points]` restores hull for credits.
- **11 – Trading Log**: `log [count]` shows recent trades and stats (v1); profit uses average cost; persists in saves.

### 📋 Planned
- **08 – Combat System** (more): other enemies, heat and power, special actions.
- **10 – Reputation System**: Factions, effects, criminal consequences.
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
- `log [count]` - Show trading history and statistics

### Travel
- `jump <system name>` - Hyperspace to another system (triggers travel events)
- `systems` - List systems in range
- `info <system name>` - Info on a system

### Ships & Upgrades
- `upgrade` - Show upgrade options
- `upgrade <ship>` - Buy a ship upgrade, e.g. `upgrade cobra4` or `upgrade Cobra Mk IV`
- `ships` - List ships for sale

### Equipment
- `equipment` - Show equipment for sale
- `equipment buy <item>` - Buy and install (replaces the current item in single-slot categories)
- `equipment sell <item>` - Sell installed equipment for half of what was paid
- `equipment installed` - Show installed equipment, slots and power
- `jobs` - Show the courier jobs on offer here
- `accept <number>` - Take a job from the board (up to 3 at once)
- `missions` - List your active jobs
- `abandon <number>` - Drop an active job (-3 reputation)
- `repair [points]` - Repair hull damage for 2 credits per point
- `refuel [units]` - Refuel for 2 credits per light year

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
Hull: [Current]/[Max]
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
│   ├── commodities.rs   # Commodities and markets, loaded from data/commodities.json
│   ├── ships.rs         # Ship types, loaded from data/ships.json
│   └── save_load.rs     # Save file I/O
└── utils/
    ├── mod.rs
    ├── display.rs       # Terminal output and formatting
    └── input.rs         # Input processing and validation
data/                    # Game data as JSON, embedded at build time (ships.json, equipment.json, commodities.json)
galaxies/                # Star system data (galaxy1.json)
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

## Releases

Releases are automatic. When a PR merges to `main`, the Release workflow reads its title (Conventional Commits) and works out the next [semver](https://semver.org) version from the latest release tag:

| PR title | Release |
|---|---|
| `feat: ...` | minor bump (0.2.0 → 0.3.0) |
| `fix: ...` | patch bump (0.2.0 → 0.2.1) |
| `feat!: ...` or any `type!:` | minor bump while below 1.0 |
| `docs`, `chore`, `ci`, `test`, `style`, `refactor` | no release |

The workflow then builds for macOS (Apple silicon and Intel), Linux and Windows, tags the merge commit, and publishes a GitHub release with the archives and notes grouped by PR label.

`Cargo.toml` holds a placeholder version (`0.0.0-dev`); the real version is set at build time, because `main` is protected and a bot cannot commit a bump. A local `cargo run` therefore reports `0.0.0-dev`.

To force a release, run the Release workflow manually from the Actions tab and pick `minor` or `patch`.

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
