# F1 CLI Simulator - Product Requirements Document

**Version:** 1.0.0  
**Date:** April 21, 2025  
**Author:** F1 CLI Simulator Development Team

## Table of Contents
1. [Introduction](#introduction)
2. [Product Overview](#product-overview)
3. [User Stories](#user-stories)
4. [Feature Requirements](#feature-requirements)
5. [Data Architecture](#data-architecture)
6. [System Architecture](#system-architecture)
7. [Technical Requirements](#technical-requirements)
8. [User Interface](#user-interface)
9. [Performance Requirements](#performance-requirements)
10. [Future Enhancements](#future-enhancements)
11. [Glossary](#glossary)

## Introduction

This document outlines the product requirements for the F1 CLI Simulator, a command-line tool for simulating Formula 1 races using both historical data and predictive models. The primary goal is to provide F1 enthusiasts with an engaging way to experience and predict race outcomes through a feature-rich command-line interface.

### Purpose

The F1 CLI Simulator aims to:
- Deliver realistic Formula 1 race experiences through the command line
- Provide access to historical F1 race data from 1950 to present
- Enable predictive modeling of future races based on statistical analysis
- Offer customizable simulation parameters for user-defined race scenarios

### Target Audience

- Formula 1 fans and enthusiasts
- Data analysts interested in F1 statistics and predictions
- Command line/terminal users who prefer text-based interfaces
- Software developers looking for an extensible F1 simulation codebase

## Product Overview

The F1 CLI Simulator is a Rust-based application that lets users interact with Formula 1 race data in multiple ways. It combines data retrieval, storage, and simulation capabilities to create an immersive racing experience without the need for graphical interfaces.

### Core Capabilities

1. **Data Management**: Fetch and store F1 race data locally
2. **Historical Simulation**: Replay actual F1 races from the past
3. **Race Prediction**: Predict outcomes of future races using statistical models
4. **Custom Simulation**: Create custom race scenarios with adjustable parameters
5. **Interactive Experience**: Lap-by-lap race playback in the terminal

## User Stories

### Data Management
- As a user, I want to download F1 data for specific seasons so I can use it offline
- As a user, I want to list available race data so I can choose what to simulate
- As a user, I want automatic data fetching when needed so I don't have to download manually

### Historical Racing
- As a user, I want to view historical race results so I can analyze past events
- As a user, I want to replay historical races interactively so I can experience them lap by lap
- As a user, I want to view qualifying and practice session data so I can understand weekend progression

### Race Prediction
- As a user, I want to predict race outcomes for upcoming events so I can anticipate future results
- As a user, I want to run multiple simulation iterations so I can get statistically significant predictions
- As a user, I want predictions to consider driver and team performance data so the results are realistic

### Custom Simulation
- As a user, I want to adjust reliability factors so I can simulate different mechanical failure scenarios
- As a user, I want to control weather conditions so I can see how they affect race outcomes
- As a user, I want to enable/disable random incidents so I can simulate chaos or clean races

## Feature Requirements

### Data Module

#### Required Features
- Fetch data from the Ergast API for seasons from 1950 to present
- Store race, qualifying, and practice data locally in JSON format
- Support for incremental updates to minimize bandwidth usage
- Automatic data validation and error handling
- Interface to query and filter available data

#### Technical Implementation
- `DataInterface` trait with concrete `DataManager` implementation
- Standardized data models for circuits, drivers, and results
- Offline-first approach with automatic downloads as needed
- Configurable data directory and caching policies

### Historical Race Module

#### Required Features
- Display race results for any historical F1 race
- Interactive lap-by-lap race replay with realistic timing
- Support for race, qualifying, and practice sessions
- Colorized terminal output for enhanced readability
- Performance statistics including fastest laps and DNF reasons

#### Technical Implementation
- `historical` module in the simulator component
- Parsing and display of race timings and positions
- Interactive mode with user-controllable playback
- Session-specific formatting and visualizations

### Prediction Module

#### Required Features
- Generate race predictions for any track on the calendar
- Support for multiple simulation runs (Monte Carlo method)
- Statistical aggregation of prediction results
- Current season driver and team performance models
- Confidence ratings for predictions

#### Technical Implementation
- `prediction` module in the simulator component
- Models for current F1 grid based on recent performance
- Driver and team performance factors
- Statistical distribution of prediction outcomes

### Simulation Module

#### Required Features
- Custom race simulations with adjustable parameters
- Reliability factor to control mechanical failures
- Weather factor to simulate different track conditions
- Random incident generation for authentic racing chaos
- Interactive and instant simulation modes
- DNF (Did Not Finish) tracking and reporting

#### Technical Implementation
- `simulation` module in the simulator component
- Random distribution models for performance variations
- Incident probability models based on input parameters
- Dynamic position calculation based on performance factors
- Lap timing simulation with realistic variability

## Data Architecture

### Data Models

The system uses several core data models to represent F1 entities:

1. **Driver**: Represents an F1 driver
   - `id`: Unique identifier string
   - `code`: Three-letter driver code (e.g., "HAM")
   - `name`: Driver's full name
   - `team`: Current team
   - `number`: Driver's chosen number

2. **Circuit**: Represents an F1 track
   - `id`: Unique identifier string
   - `name`: Circuit name
   - `country`: Country location
   - `city`: City location
   - `length_km`: Track length in kilometers
   - `laps`: Standard number of laps

3. **Race**: Represents a race event
   - `season`: Year of the race
   - `round`: Race number in the season
   - `name`: Grand Prix name
   - `circuit`: Associated Circuit
   - `date`: Race date
   - `results`: Array of RaceResult objects

4. **RaceResult**: Represents a driver's race outcome
   - `position`: Finishing position
   - `driver`: Associated Driver
   - `time`: Finish time (or null for DNF)
   - `points`: Championship points earned
   - `laps`: Laps completed
   - `status`: Finishing status (e.g., "Finished", "Collision", etc.)

5. **QualifyingResult**: Represents qualifying performance
   - `position`: Grid position
   - `driver`: Associated Driver
   - `q1`: Q1 session time
   - `q2`: Q2 session time
   - `q3`: Q3 session time

6. **PracticeResult**: Represents practice session performance
   - `position`: Position in the session
   - `driver`: Associated Driver
   - `time`: Best lap time
   - `laps`: Laps completed

7. **SimulationParameters**: Controls custom race simulations
   - `reliability_factor`: Factor affecting mechanical failures (0.5-1.5)
   - `weather_factor`: Factor affecting wet/dry conditions (0.7-1.2)
   - `random_incidents`: Boolean to enable/disable random racing incidents

### Data Storage

- All race data is stored in JSON format in the `./data/` directory
- Season data in `season_YYYY.json` files
- Race data in `race_YYYY_circuit.json` files
- Implementation leverages the `serde` and `serde_json` crates for serialization

## System Architecture

### High-Level Architecture

The F1 CLI Simulator follows a modular design pattern with clear separation of concerns:

1. **Command-Line Interface**
   - Uses `clap` for argument parsing and command definition
   - Provides commands for historical simulation, prediction, and custom simulation

2. **Data Management**
   - Handles data retrieval, storage, and querying
   - Implements caching mechanisms for efficient operation

3. **Simulation Engines**
   - Historical: Replays actual races
   - Prediction: Predicts race outcomes using statistical models
   - Custom: Simulates races with user-defined parameters

### Component Diagram
```
[CLI (main.rs)] -- parses commands --> [Data Module (data.rs)]
                                     |
                                     +--> [Historical Module (historical.rs)]
                                     |
                                     +--> [Prediction Module (prediction.rs)]
                                     |
                                     +--> [Simulation Module (simulation.rs)]
```

### Module Structure

- **src/main.rs**: Entry point with CLI command definitions
- **src/lib.rs**: Library interface for external usage
- **src/data.rs**: Data management functionality
- **src/models.rs**: Data models and structures
- **src/utils.rs**: Shared utility functions
- **src/simulator.rs**: Parent module for simulation engines
  - **src/simulator/historical.rs**: Historical race simulation
  - **src/simulator/prediction.rs**: Race outcome prediction
  - **src/simulator/simulation.rs**: Custom race simulation

## Technical Requirements

### Development Environment

- **Language**: Rust 1.70.0 or newer
- **Build System**: Cargo
- **Target Platforms**: macOS, Linux, Windows

### Dependencies

- **clap**: Command line argument parsing
- **reqwest**: HTTP client for API requests
- **serde/serde_json**: JSON serialization/deserialization
- **rand/rand_distr**: Random number generation and distributions
- **colored**: Terminal text coloring
- **indicatif**: Progress indicators and spinners
- **anyhow**: Error handling

### Testing Strategy

- Unit tests for core functionality
- Mock objects for external dependencies
- Integration tests for end-to-end workflows
- Test coverage for major simulation algorithms
- Performance benchmarks for simulation speed

## User Interface

The F1 CLI Simulator uses a command-line interface with the following structure:

### Base Command

```
f1-cli-simulator [COMMAND] [OPTIONS]
```

### Available Commands

1. **historical**: Access historical race data
   ```
   f1-cli-simulator historical --season <YEAR> --gp <NAME> [--session <TYPE>] [--interactive]
   ```

2. **predict**: Predict future race outcomes
   ```
   f1-cli-simulator predict --season <YEAR> --gp <NAME> [--runs <NUMBER>]
   ```

3. **simulate**: Run custom race simulations
   ```
   f1-cli-simulator simulate --season <YEAR> --gp <NAME> [--reliability <FACTOR>] [--weather <FACTOR>] [--no-incidents] [--interactive]
   ```

4. **list**: Show available race data
   ```
   f1-cli-simulator list [--season <YEAR>]
   ```

5. **update**: Update local data cache
   ```
   f1-cli-simulator update [--previous <NUMBER>] [--seasons <LIST>] [--all]
   ```

### Output Format

- Color-coded terminal output for improved readability
- Tabular data display for race results
- Progress indicators for long-running operations
- Clear error messages for better user guidance
- Interactive playback with realistic timing

## Performance Requirements

- Data download and parsing should complete within reasonable timeframes
- Prediction runs should process at least 10 simulations per second
- Custom simulations should run at interactive speeds
- Memory usage should remain under 100MB for standard operations
- Application startup time should be under 1 second

## Future Enhancements

### Short-term Roadmap (Next 3-6 Months)

1. **Championship Simulation**
   - Simulate entire seasons and generate championship standings
   - Support for driver and constructor championships

2. **Enhanced Statistical Models**
   - More advanced modeling of driver performance
   - Circuit-specific performance factors
   - Car development progression throughout seasons

3. **Team Strategy Simulation**
   - Pit stop strategy optimization
   - Tire compound selection modeling
   - Team orders simulation

### Long-term Vision (6-12 Months)

1. **Web API Integration**
   - Provide a REST API for simulation functionality
   - Enable integration with web applications

2. **Enhanced Visualization**
   - Text-based track maps showing driver positions
   - ASCII art representation of race standings
   - More detailed lap-by-lap analysis

3. **Historical Analysis**
   - "What-if" scenarios for historical races
   - Driver performance comparison across eras
   - Team development trajectory analysis

## Glossary

- **DNF**: Did Not Finish - A race result where the driver could not complete the full race distance
- **GP**: Grand Prix - A Formula 1 race event
- **FP1/FP2/FP3**: Free Practice sessions 1, 2, and 3
- **Q1/Q2/Q3**: Qualifying sessions 1, 2, and 3
- **Reliability Factor**: A simulation parameter affecting mechanical failures
- **Weather Factor**: A simulation parameter affecting track conditions
- **Ergast API**: External data source for Formula 1 statistics