# F1 CLI Simulator

A command-line tool for simulating Formula 1 races and sessions, fetching real F1 data, and running predictive race simulations.

![F1 CLI Simulator Banner](https://placehold.co/800x200/0078D7/FFFFFF?text=F1+CLI+Simulator)

## Features

- **Historical Race Data**: Access real F1 race results from the Ergast API
- **Interactive Race Simulation**: Simulate F1 races lap-by-lap with realistic parameters and events
- **Auto-Fetching**: Automatically downloads race data when requested if not available locally
- **Predictive Analysis**: Run multiple simulations to predict race outcomes and driver performance
- **Data Management**: Download and manage race data for offline use
- **Rich Terminal Output**: Colored and formatted race results with fastest laps, DNFs and more

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or newer)

### Building from Source

Clone the repository and build with Cargo:

```bash
git clone https://github.com/yourusername/f1-cli-simulator.git
cd f1-cli-simulator
cargo build --release
```

The executable will be available at `target/release/f1-cli-simulator`.

## Usage

### Basic Commands

```bash
# Update F1 data from the internet
f1-cli-simulator update

# List available races
f1-cli-simulator list

# List races for a specific season
f1-cli-simulator list --season 2023

# View historical race results
f1-cli-simulator historical --gp monza --season 2023 --session race

# Run statistical predictions for a race
f1-cli-simulator predict --gp monaco --season 2025 --runs 100

# Experience an interactive race simulation
f1-cli-simulator simulate --gp spa --season 2025 --interactive --weather 0.8

# Get help
f1-cli-simulator --help
```

### Command Options

#### `update`
Downloads F1 data from the Ergast API.

#### `list`
Lists available race data.
- `--season <YEAR>`: Filter by season year

#### `historical`
Shows historical race data.
- `--gp <NAME>`, `-g <NAME>`: Grand Prix name (e.g., "monza", "monaco", "spa")
- `--season <YEAR>`, `-s <YEAR>`: Season year
- `--session <TYPE>`, `-t <TYPE>`: Session type ("race", "qualifying", "practice", "fp1", "fp2" or "fp3")

#### `predict`
Runs multiple race simulations to predict outcomes.
- `--gp <NAME>`, `-g <NAME>`: Grand Prix name
- `--season <YEAR>`, `-s <YEAR>`: Season year
- `--runs <NUMBER>`, `-r <NUMBER>`: Number of simulation runs (default: 100)

#### `simulate`
Runs an interactive or instant race simulation with customizable parameters.
- `--gp <NAME>`, `-g <NAME>`: Grand Prix name
- `--season <YEAR>`, `-s <YEAR>`: Season year
- `--reliability <FACTOR>`, `-r <FACTOR>`: Reliability factor (0.5-1.5, higher means fewer failures, default: 0.95)
- `--weather <FACTOR>`, `-w <FACTOR>`: Weather factor (0.7-1.2, lower means wetter conditions, default: 1.0)
- `--no-incidents`, `-n`: Disable random racing incidents
- `--interactive`, `-i`: Run in interactive mode with lap-by-lap updates

## Examples

### View the results of a historical race

```bash
f1-cli-simulator historical --gp silverstone --season 2023 --session race
# or using short options
f1-cli-simulator historical -g silverstone -s 2023 -t race
```

### Run a statistical prediction for an upcoming race

```bash
f1-cli-simulator predict --gp monaco --season 2025 --runs 500
```

### Experience an interactive race simulation with wet weather

```bash
f1-cli-simulator simulate --gp spa --season 2025 --interactive --weather 0.8
```

### Run a quick race simulation with high reliability (fewer failures)

```bash
f1-cli-simulator simulate --gp monza --season 2025 --reliability 1.2 --no-incidents
```

### Update the local F1 data cache

```bash
f1-cli-simulator update
```

## Data Sources

This application uses the [Ergast Developer API](http://ergast.com/mrd/) to fetch Formula 1 race data. The data is stored locally for offline use after the initial download.

## Technical Details

The F1 CLI Simulator is built with Rust and uses several key libraries:
- `clap` for command line argument parsing
- `reqwest` for API requests
- `serde` and `serde_json` for JSON handling
- `rand` and `rand_distr` for simulation randomization
- `colored` for terminal output formatting
- `indicatif` for progress indicators

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This project is not affiliated with Formula 1, FIA, or any F1 team. All Formula 1 related trademarks belong to their respective owners.