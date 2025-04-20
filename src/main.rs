use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod data;
mod simulator;
mod utils;
mod models;

#[derive(Parser)]
#[command(name = "f1-cli-simulator")]
#[command(about = "Formula 1 Race Simulator CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Simulate a historical F1 race using actual race data
    Historical {
        /// Season year (e.g., 2023)
        #[arg(short, long)]
        season: u32,
        
        /// GP name (e.g., "monaco", "spa", "monza")
        #[arg(short, long)]
        gp: String,
        
        /// Session type: "practice", "qualifying", or "race"
        #[arg(short = 't', long, default_value = "race")]
        session: String,
    },
    
    /// Simulate an upcoming F1 race using predictive modeling
    Predict {
        /// Season year (e.g., 2025)
        #[arg(short, long)]
        season: u32,
        
        /// GP name (e.g., "monaco", "spa", "monza")
        #[arg(short, long)]
        gp: String,
        
        /// Number of simulation runs to aggregate results from
        #[arg(short, long, default_value_t = 100)]
        runs: u32,
    },
    
    /// Simulate a custom F1 race with adjustable parameters
    Simulate {
        /// Season year (e.g., 2025)
        #[arg(short, long)]
        season: u32,
        
        /// GP name (e.g., "monaco", "spa", "monza")
        #[arg(short, long)]
        gp: String,
        
        /// Reliability factor (0.5-1.5, where higher means fewer mechanical failures)
        #[arg(short = 'r', long, default_value_t = 0.95)]
        reliability: f64,
        
        /// Weather factor (0.7-1.2, where lower means wetter conditions)
        #[arg(short = 'w', long, default_value_t = 1.0)]
        weather: f64,
        
        /// Disable random racing incidents
        #[arg(short = 'n', long)]
        no_incidents: bool,
        
        /// Run in interactive mode (lap-by-lap updates)
        #[arg(short, long)]
        interactive: bool,
    },
    
    /// List available historical race data
    List {
        /// Filter by season year (optional)
        #[arg(short, long)]
        season: Option<u32>,
    },
    
    /// Update the local database of F1 race data
    Update {
        /// Number of previous seasons to fetch (in addition to current season)
        #[arg(short, long)]
        previous: Option<u32>,
        
        /// Specific comma-separated seasons to fetch (e.g., "2010,2015,2020")
        #[arg(short, long)]
        seasons: Option<String>,
        
        /// Fetch all historical seasons (from 1950 to current)
        #[arg(short, long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    println!("{}", "F1 Race Simulator CLI".bright_green().bold());
    println!("{}", "------------------------".bright_green());
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Historical { season, gp, session } => {
            println!("Simulating historical {} session for {} GP {}", session, gp, season);
            simulator::historical::simulate(season, &gp, &session)
        },
        Commands::Predict { season, gp, runs } => {
            println!("Predicting {} GP {} with {} simulation runs", gp, season, runs);
            simulator::prediction::simulate(season, &gp, runs)
        },
        Commands::Simulate { season, gp, reliability, weather, no_incidents, interactive } => {
            println!("Simulating custom race for {} GP {} with reliability {}, weather {}, no incidents: {}, interactive: {}", 
                     gp, season, reliability, weather, no_incidents, interactive);
            
            let params = models::SimulationParameters {
                reliability_factor: reliability,
                weather_factor: weather,
                random_incidents: !no_incidents,
            };
            
            simulator::simulation::simulate(season, &gp, params, interactive)
        },
        Commands::List { season } => {
            match season {
                Some(year) => println!("Listing available race data for season {}", year),
                None => println!("Listing all available race data"),
            }
            data::list_available_data(season)
        },
        Commands::Update { previous, seasons, all } => {
            println!("Updating F1 race data...");
            data::update_data(previous, seasons, all)
        },
    }
}
