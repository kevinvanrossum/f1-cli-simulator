use anyhow::Result;
use std::time::Duration;
use std::collections::HashMap;
use std::thread;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rand_distr::{Normal, Distribution};

use crate::models::{Driver, Circuit, RaceResult, SimulationParameters};
use crate::utils;

// Helper function to multiply Duration by a float
#[allow(dead_code)]
fn mul_f64(duration: Duration, factor: f64) -> Duration {
    let nanos = duration.as_nanos() as f64 * factor;
    Duration::from_nanos(nanos as u64)
}

// Extension trait to add mul_f64 method to Duration
#[allow(dead_code)]
trait DurationExt {
    fn mul_f64(&self, factor: f64) -> Duration;
}

impl DurationExt for Duration {
    fn mul_f64(&self, factor: f64) -> Duration {
        let nanos = self.as_nanos() as f64 * factor;
        Duration::from_nanos(nanos as u64)
    }
}

const CURRENT_DRIVERS: [(&str, &str, u32); 20] = [
    ("VER", "Max Verstappen", 1),
    ("PER", "Sergio Perez", 11),
    ("LEC", "Charles Leclerc", 16),
    ("SAI", "Carlos Sainz", 55),
    ("HAM", "Lewis Hamilton", 44),
    ("RUS", "George Russell", 63),
    ("NOR", "Lando Norris", 4),
    ("PIA", "Oscar Piastri", 81),
    ("ALO", "Fernando Alonso", 14),
    ("STR", "Lance Stroll", 18),
    ("GAS", "Pierre Gasly", 10),
    ("OCO", "Esteban Ocon", 31),
    ("ALB", "Alexander Albon", 23),
    ("SAR", "Logan Sargeant", 2),
    ("TSU", "Yuki Tsunoda", 22),
    ("LAW", "Liam Lawson", 40),
    ("MAG", "Kevin Magnussen", 20),
    ("HUL", "Nico Hulkenberg", 27),
    ("BOT", "Valtteri Bottas", 77),
    ("ZHO", "Guanyu Zhou", 24),
];

const CURRENT_TEAMS: [&str; 10] = [
    "Red Bull Racing",
    "Ferrari",
    "Mercedes",
    "McLaren",
    "Aston Martin",
    "Alpine",
    "Williams",
    "RB",
    "Haas F1 Team",
    "Sauber",
];

/// Simulate a race with predictive modeling
pub fn simulate(season: u32, gp: &str, runs: u32) -> Result<()> {
    println!("{}", format!("Predicting {} GP {} with {} simulation runs", gp, season, runs).blue());
    
    // Set up progress bar for simulation runs
    let pb = ProgressBar::new(runs as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} runs ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    // Create a circuit for the specified GP
    let circuit = create_circuit_for_gp(gp)?;
    
    // Create current drivers
    let drivers = create_current_drivers();
    
    // Initialize simulation parameters
    let params = SimulationParameters::default();
    
    // Prepare to collect aggregated results from all simulation runs
    let mut position_counts: HashMap<String, HashMap<u32, u32>> = HashMap::new();
    let mut dnf_counts: HashMap<String, u32> = HashMap::new();
    let mut points_totals: HashMap<String, f64> = HashMap::new();
    let mut win_count: HashMap<String, u32> = HashMap::new();
    let mut podium_count: HashMap<String, u32> = HashMap::new();
    
    // Run the simulations
    for _ in 0..runs {
        let race_results = run_single_simulation(&drivers, &circuit, &params);
        
        // Aggregate results
        for result in &race_results {
            let driver_name = &result.driver.name;
            
            // Count positions
            let positions = position_counts.entry(driver_name.clone()).or_insert_with(HashMap::new);
            *positions.entry(result.position).or_insert(0) += 1;
            
            // Count DNFs
            if result.status != "Finished" {
                *dnf_counts.entry(driver_name.clone()).or_insert(0) += 1;
            }
            
            // Sum points
            *points_totals.entry(driver_name.clone()).or_insert(0.0) += result.points as f64;
            
            // Count wins and podiums
            if result.position == 1 {
                *win_count.entry(driver_name.clone()).or_insert(0) += 1;
            }
            
            if result.position <= 3 {
                *podium_count.entry(driver_name.clone()).or_insert(0) += 1;
            }
        }
        
        pb.inc(1);
        
        // Small delay to make the simulation look more realistic
        thread::sleep(Duration::from_millis(10));
    }
    
    pb.finish_with_message("Simulation completed!");
    
    // Calculate average points and winning probabilities
    let mut driver_stats: Vec<(String, f64, f64, f64)> = drivers.iter().map(|d| {
        let name = &d.name;
        let avg_points = *points_totals.get(name).unwrap_or(&0.0) / runs as f64;
        let win_prob = *win_count.get(name).unwrap_or(&0) as f64 / runs as f64;
        let podium_prob = *podium_count.get(name).unwrap_or(&0) as f64 / runs as f64;
        
        (name.clone(), avg_points, win_prob, podium_prob)
    }).collect();
    
    // Sort by average points
    driver_stats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Display prediction results
    display_prediction_results(gp, season, runs, driver_stats);
    
    Ok(())
}

/// Create a circuit model for the specified GP
pub fn create_circuit_for_gp(gp: &str) -> Result<Circuit> {
    // Normalize the GP name
    let normalized_gp = utils::normalize_gp_name(gp);
    
    // Circuit characteristics - in a real app this would come from a database
    let (name, country, city, length_km, laps) = match normalized_gp.as_str() {
        "monaco" => ("Circuit de Monaco", "Monaco", "Monte Carlo", 3.337, 78),
        "monza" | "italian" => ("Autodromo Nazionale Monza", "Italy", "Monza", 5.793, 53),
        "spa" | "belgian" => ("Circuit de Spa-Francorchamps", "Belgium", "Spa", 7.004, 44),
        "silverstone" | "british" => ("Silverstone Circuit", "UK", "Silverstone", 5.891, 52),
        "catalunya" | "spanish" => ("Circuit de Barcelona-Catalunya", "Spain", "Barcelona", 4.675, 66),
        "albert_park" | "australian" => ("Albert Park Circuit", "Australia", "Melbourne", 5.278, 58),
        "villeneuve" | "canadian" => ("Circuit Gilles Villeneuve", "Canada", "Montreal", 4.361, 70),
        "baku" | "azerbaijan" => ("Baku City Circuit", "Azerbaijan", "Baku", 6.003, 51),
        "hungaroring" | "hungarian" => ("Hungaroring", "Hungary", "Budapest", 4.381, 70),
        "suzuka" | "japanese" => ("Suzuka International Racing Course", "Japan", "Suzuka", 5.807, 53),
        "marina_bay" | "singapore" => ("Marina Bay Street Circuit", "Singapore", "Singapore", 5.063, 61),
        "americas" | "us" => ("Circuit of the Americas", "USA", "Austin", 5.513, 56),
        "rodriguez" | "mexican" => ("Autódromo Hermanos Rodríguez", "Mexico", "Mexico City", 4.304, 71),
        "interlagos" | "brazilian" => ("Autódromo José Carlos Pace", "Brazil", "São Paulo", 4.309, 71),
        "yas_marina" | "abu_dhabi" => ("Yas Marina Circuit", "UAE", "Abu Dhabi", 5.554, 55),
        "bahrain" => ("Bahrain International Circuit", "Bahrain", "Sakhir", 5.412, 57),
        "jeddah" | "saudi" => ("Jeddah Corniche Circuit", "Saudi Arabia", "Jeddah", 6.174, 50),
        "imola" => ("Autodromo Enzo e Dino Ferrari", "Italy", "Imola", 4.909, 63),
        "miami" => ("Miami International Autodrome", "USA", "Miami", 5.412, 57),
        "zandvoort" | "dutch" => ("Circuit Zandvoort", "Netherlands", "Zandvoort", 4.259, 72),
        "las_vegas" => ("Las Vegas Strip Circuit", "USA", "Las Vegas", 6.12, 50),
        "losail" | "qatar" => ("Losail International Circuit", "Qatar", "Lusail", 5.38, 57),
        _ => return Err(anyhow::anyhow!("Unknown GP: {}", gp)),
    };
    
    Ok(Circuit {
        id: normalized_gp,
        name: name.to_string(),
        country: country.to_string(),
        city: city.to_string(),
        length_km,
        laps,
    })
}

/// Create a list of current F1 drivers
pub fn create_current_drivers() -> Vec<Driver> {
    let mut drivers = Vec::new();
    
    for (i, &(code, name, number)) in CURRENT_DRIVERS.iter().enumerate() {
        let team = CURRENT_TEAMS[i / 2]; // Assign 2 drivers per team
        
        drivers.push(Driver {
            id: code.to_lowercase(),
            code: code.to_string(),
            name: name.to_string(),
            team: team.to_string(),
            number,
        });
    }
    
    drivers
}

/// Run a single race simulation
fn run_single_simulation(
    drivers: &[Driver],
    circuit: &Circuit, 
    params: &SimulationParameters
) -> Vec<RaceResult> {
    let mut rng = rand::thread_rng();
    let mut results = Vec::new();
    
    // Driver skill factors (simplified model)
    let driver_skill: HashMap<&str, f64> = [
        ("Max Verstappen", 0.98),
        ("Sergio Perez", 0.92),
        ("Charles Leclerc", 0.95),
        ("Carlos Sainz", 0.94),
        ("Lewis Hamilton", 0.96),
        ("George Russell", 0.94),
        ("Lando Norris", 0.96),
        ("Oscar Piastri", 0.93),
        ("Fernando Alonso", 0.95),
        ("Lance Stroll", 0.90),
    ].iter().cloned().collect();
    
    // Team performance factors (simplified model)
    let team_performance: HashMap<&str, f64> = [
        ("Red Bull Racing", 0.98),
        ("Ferrari", 0.96),
        ("Mercedes", 0.95),
        ("McLaren", 0.97),
        ("Aston Martin", 0.92),
        ("Alpine", 0.89),
        ("Williams", 0.87),
        ("RB", 0.88),
        ("Haas F1 Team", 0.86),
        ("Sauber", 0.85),
    ].iter().cloned().collect();
    
    // Calculate base performance for each driver
    let mut driver_performances: Vec<(usize, f64, Duration)> = Vec::new();
    
    for (i, driver) in drivers.iter().enumerate() {
        // Get driver skill and team performance
        let skill = *driver_skill.get(driver.name.as_str()).unwrap_or(&0.90);
        let team_perf = *team_performance.get(driver.team.as_str()).unwrap_or(&0.85);
        
        // Calculate base performance - higher is better
        let base_performance = skill * team_perf;
        
        // Add random variation for a single race
        let race_variation = Normal::new(0.0, 0.03).unwrap();
        let perf_variation = 1.0 + race_variation.sample(&mut rng);
        let race_performance = base_performance * perf_variation;
        
        // Convert performance to race time
        // Lower performance = longer time (worse)
        let base_lap_time = Duration::from_secs_f64(90.0); // Average lap time of 1:30
        let performance_factor = 1.0 + (1.0 - race_performance) * 0.2; // Max 20% slower
        let average_lap_time = base_lap_time.mul_f64(performance_factor);
        
        let total_race_time = average_lap_time.mul_f64(circuit.laps as f64);
        
        driver_performances.push((i, race_performance, total_race_time));
    }
    
    // Simulate mechanical failures and incidents
    let mut dnf_drivers = Vec::new();
    
    if params.random_incidents {
        for (i, driver) in drivers.iter().enumerate() {
            if utils::simulate_mechanical_failure(driver, params.reliability_factor) {
                dnf_drivers.push(i);
            }
        }
    }
    
    // Sort by race time (faster times first)
    driver_performances.sort_by(|a, b| a.2.cmp(&b.2));
    
    // Create race results
    for (position, (driver_idx, _, total_time)) in driver_performances.iter().enumerate() {
        let position = (position + 1) as u32;
        let driver = drivers[*driver_idx].clone();
        
        let (time, status, points, laps_completed) = if dnf_drivers.contains(driver_idx) {
            // DNF - calculate random lap for the incident
            let max_laps = circuit.laps;
            let incident_lap = rng.gen_range((max_laps / 3)..(max_laps - 3));
            let status = utils::get_random_incident().to_string();
            
            (None, status, 0, incident_lap)
        } else {
            // Regular finish
            let time_str = format!("{}:{:02}.{:03}", 
                total_time.as_secs() / 60,
                total_time.as_secs() % 60,
                total_time.subsec_millis()
            );
            
            // Calculate points
            let points = match position {
                1 => 25,
                2 => 18,
                3 => 15,
                4 => 12,
                5 => 10,
                6 => 8,
                7 => 6,
                8 => 4,
                9 => 2,
                10 => 1,
                _ => 0,
            };
            
            (Some(time_str), "Finished".to_string(), points, circuit.laps)
        };
        
        results.push(RaceResult {
            position,
            driver,
            time,
            points,
            laps: laps_completed,
            status,
        });
    }
    
    results
}

/// Display prediction results
fn display_prediction_results(
    gp: &str, 
    season: u32, 
    runs: u32,
    driver_stats: Vec<(String, f64, f64, f64)>
) {
    println!("\n{} {}", 
        format!("Prediction Results for {} GP {}", gp, season).green().bold(),
        format!("(based on {} simulations)", runs).italic()
    );
    
    println!("{}", "-".repeat(70));
    
    println!("{:<3} {:<20} {:<15} {:<15} {}", 
        "Pos".bold(), 
        "Driver".bold(), 
        "Avg Points".bold(),
        "Win Chance".bold(),
        "Podium Chance".bold()
    );
    
    println!("{}", "-".repeat(70));
    
    for (i, (name, avg_points, win_prob, podium_prob)) in driver_stats.iter().enumerate() {
        let position = i + 1;
        let position_str = format!("{}", position);
        let position_colored = match position {
            1 => position_str.bright_yellow(),
            2 => position_str.bright_white(),
            3 => position_str.yellow(),
            _ => position_str.normal(),
        };
        
        println!("{:<3} {:<20} {:<15.2} {:<15.1}% {:.1}%",
            position_colored,
            name,
            avg_points,
            win_prob * 100.0,
            podium_prob * 100.0
        );
    }
    
    println!("\n{}", "Note: These predictions are simulations based on estimated data.".italic());
}