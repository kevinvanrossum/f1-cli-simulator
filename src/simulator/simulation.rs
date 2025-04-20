use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rand_distr::{Normal, Distribution};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crate::models::{Circuit, Driver, SimulationParameters};
use crate::simulator::prediction::{create_circuit_for_gp, create_current_drivers};
use crate::utils;

/// Simulate a race with customizable parameters
pub fn simulate(season: u32, gp: &str, params: SimulationParameters, interactive: bool) -> Result<()> {
    println!("{}", format!("Simulating {} GP {}", gp, season).blue());
    println!("Simulation parameters:");
    println!("  - Reliability factor: {:.2}", params.reliability_factor);
    println!("  - Weather factor: {:.2}", params.weather_factor);
    println!("  - Random incidents: {}", params.random_incidents);
    
    // Create a circuit for the specified GP
    let circuit = create_circuit_for_gp(gp)?;
    
    // Create current drivers
    let drivers = create_current_drivers();
    
    if interactive {
        simulate_interactive_race(&drivers, &circuit, &params)
    } else {
        simulate_instant_race(&drivers, &circuit, &params)
    }
}

/// Run a single race simulation with turn-by-turn interactive display
pub fn simulate_interactive_race(drivers: &[Driver], circuit: &Circuit, params: &SimulationParameters) -> Result<()> {
    println!("\n{}", format!("Interactive Race Simulation at {}", circuit.name).green().bold());
    println!("{} laps, {:.3} km", circuit.laps, circuit.length_km);
    println!("{}","-".repeat(50));
    
    println!("\n{}", "Starting Grid:".yellow());
    // Show the starting grid (we'll randomize it a bit)
    let mut driver_positions = initialize_driver_positions(drivers, params);
    
    for (pos, (idx, _, _, _)) in driver_positions.iter().enumerate() {
        let driver = &drivers[*idx];
        println!("{:2}. {} - {}", pos + 1, driver.code, driver.team);
    }
    
    println!("\n{}", "Press Enter to start the race...".green());
    wait_for_user_input();
    
    let total_laps = circuit.laps;
    let mut dnf_drivers = Vec::new();
    let mut fastest_lap: Option<(usize, Duration)> = None;
    
    // Initialize lap times with some baseline performance
    let mut driver_performance = HashMap::new();
    for (i, driver) in drivers.iter().enumerate() {
        // Base performance on a combination of driver skill and car performance
        let base_performance = calculate_driver_base_performance(driver, params);
        driver_performance.insert(i, base_performance);
    }
    
    // Run the race lap by lap
    for lap in 1..=total_laps {
        println!("\n{}", format!("Lap {}/{}", lap, total_laps).bold());
        
        // Update positions and handle incidents
        update_race_positions(&mut driver_positions, &driver_performance, params);
        
        // Check for incidents/DNFs
        if params.random_incidents && lap > 5 {
            check_for_incidents(drivers, &mut driver_positions, &mut dnf_drivers, lap, params);
        }
        
        // Display current positions (top 5)
        display_lap_summary(drivers, &driver_positions, lap, &dnf_drivers, fastest_lap);
        
        if lap < total_laps {
            // Interactive mode - wait for user to continue
            if lap % 10 == 0 || lap == total_laps - 1 {
                println!("\nPress Enter to continue...");
                wait_for_user_input();
            } else {
                // Short delay between laps for race feel
                thread::sleep(Duration::from_millis(800));
            }
        }
        
        // Update fastest lap
        update_fastest_lap(&driver_positions, lap, &mut fastest_lap);
    }
    
    // Show final results
    display_final_results(drivers, &driver_positions, &dnf_drivers, fastest_lap);
    
    Ok(())
}

/// Run a race simulation and display the final results immediately
pub fn simulate_instant_race(drivers: &[Driver], circuit: &Circuit, params: &SimulationParameters) -> Result<()> {
    println!("\n{}", format!("Race Simulation at {}", circuit.name).green().bold());
    println!("{} laps, {:.3} km", circuit.laps, circuit.length_km);
    println!("{}","-".repeat(50));
    
    // Set up progress bar for simulation
    let pb = ProgressBar::new(circuit.laps as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} laps ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    // Initialize positions and performance
    let mut driver_positions = initialize_driver_positions(drivers, params);
    let mut dnf_drivers = Vec::new();
    let mut fastest_lap: Option<(usize, Duration)> = None;
    
    // Initialize driver performance
    let mut driver_performance = HashMap::new();
    for (i, driver) in drivers.iter().enumerate() {
        let base_performance = calculate_driver_base_performance(driver, params);
        driver_performance.insert(i, base_performance);
    }
    
    // Run the simulation
    for lap in 1..=circuit.laps {
        // Update positions
        update_race_positions(&mut driver_positions, &driver_performance, params);
        
        // Check for incidents
        if params.random_incidents && lap > 5 {
            check_for_incidents(drivers, &mut driver_positions, &mut dnf_drivers, lap, params);
        }
        
        // Update fastest lap
        update_fastest_lap(&driver_positions, lap, &mut fastest_lap);
        
        pb.inc(1);
        thread::sleep(Duration::from_millis(10)); // Small delay for visual effect
    }
    
    pb.finish_with_message("Race completed!");
    
    // Display final results
    display_final_results(drivers, &driver_positions, &dnf_drivers, fastest_lap);
    
    Ok(())
}

// Initialize driver positions with qualifying performance
pub fn initialize_driver_positions(drivers: &[Driver], params: &SimulationParameters) -> Vec<(usize, f64, Duration, bool)> {
    let mut rng = rand::thread_rng();
    let mut positions = Vec::new();
    
    for (i, driver) in drivers.iter().enumerate() {
        // Base performance calculation
        let base_perf = calculate_driver_base_performance(driver, params);
        
        // Add qualifying variation
        let quali_variation = Normal::new(0.0, 0.015).unwrap();
        let perf_variation = 1.0 + quali_variation.sample(&mut rng);
        let quali_performance = base_perf * perf_variation;
        
        // Convert performance to time
        let base_lap_time = Duration::from_secs_f64(90.0);
        let performance_factor = 1.0 + (1.0 - quali_performance) * 0.15;
        let quali_time = base_lap_time.mul_f64(performance_factor);
        
        positions.push((i, quali_performance, quali_time, true));
    }
    
    // Sort by qualifying time (lower is better)
    positions.sort_by(|a, b| a.2.cmp(&b.2));
    
    positions
}

// Calculate base performance for a driver (0-1 scale, higher is better)
pub fn calculate_driver_base_performance(driver: &Driver, params: &SimulationParameters) -> f64 {
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
        // Add more drivers as needed
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
    
    // Get driver skill and team performance
    let skill = *driver_skill.get(driver.name.as_str()).unwrap_or(&0.90);
    let team_perf = *team_performance.get(driver.team.as_str()).unwrap_or(&0.85);
    
    // Apply weather factor for wet weather variance
    let weather_adjustment = if params.weather_factor < 1.0 {
        // Wet conditions can shuffle the order slightly
        // Ensure weather adjustment is always between 0.7 and 1.0
        // This ensures the final performance value stays within reasonable bounds
        0.7 + (params.weather_factor * 0.3)
    } else {
        1.0
    };
    
    // Final base performance
    skill * team_perf * weather_adjustment
}

// Update race positions for the current lap
pub fn update_race_positions(
    positions: &mut Vec<(usize, f64, Duration, bool)>, 
    driver_performance: &HashMap<usize, f64>,
    params: &SimulationParameters
) {
    let mut rng = rand::thread_rng();
    
    // For each driver still in the race
    for i in 0..positions.len() {
        if !positions[i].3 {
            continue; // Skip DNF'd drivers
        }
        
        let driver_idx = positions[i].0;
        let base_perf = *driver_performance.get(&driver_idx).unwrap_or(&0.9);
        
        // Add lap-to-lap variation
        let lap_variation = Normal::new(0.0, 0.01 * params.weather_factor).unwrap();
        let variation = 1.0 + lap_variation.sample(&mut rng);
        
        // Adjust performance for this lap
        let lap_performance = base_perf * variation;
        positions[i].1 = lap_performance;
        
        // Attempt overtake logic
        if i > 0 && positions[i].3 && positions[i-1].3 {
            let overtake_chance = (positions[i].1 - positions[i-1].1) * 2.5;
            if overtake_chance > 0.0 && rng.gen::<f64>() < overtake_chance {
                // Successful overtake
                positions.swap(i, i-1);
            }
        }
    }
}

// Check for mechanical failures and incidents
pub fn check_for_incidents(
    drivers: &[Driver], 
    positions: &mut Vec<(usize, f64, Duration, bool)>,
    dnf_drivers: &mut Vec<usize>,
    current_lap: u32,
    params: &SimulationParameters
) {
    let mut rng = rand::thread_rng();
    
    // Using underscore prefix to suppress the unused variable warning
    for (_race_pos, (driver_idx, _, _, active)) in positions.iter_mut().enumerate() {
        // Skip already DNF'd drivers
        if !*active || dnf_drivers.contains(driver_idx) {
            continue;
        }
        
        let driver = &drivers[*driver_idx];
        
        // Check for mechanical failure
        if utils::simulate_mechanical_failure(driver, params.reliability_factor) {
            // This driver has a mechanical failure
            *active = false;
            dnf_drivers.push(*driver_idx);
            
            // Print the incident
            println!("\n{}", format!("LAP {} - INCIDENT: {} (#{}) - {}", 
                current_lap, 
                driver.name,
                driver.number,
                utils::get_random_incident()
            ).red());
        }
        
        // Check for racing incidents (more likely in wet conditions)
        let incident_factor = if params.weather_factor < 0.8 { 3.0 } else { 1.0 };
        let incident_chance = 0.0005 * incident_factor / params.reliability_factor;
        
        if rng.gen::<f64>() < incident_chance {
            // Racing incident
            *active = false;
            dnf_drivers.push(*driver_idx);
            
            // Print the incident
            println!("\n{}", format!("LAP {} - INCIDENT: {} (#{}) crashed!", 
                current_lap, 
                driver.name,
                driver.number
            ).red());
        }
    }
}

// Update the fastest lap record
pub fn update_fastest_lap(
    positions: &Vec<(usize, f64, Duration, bool)>,
    lap: u32,
    fastest_lap: &mut Option<(usize, Duration)>
) {
    // For each active driver, generate a lap time
    for &(driver_idx, perf, _, active) in positions.iter() {
        if !active {
            continue;
        }
        
        // Generate a lap time based on performance
        let base_time = Duration::from_secs_f64(90.0); // 1:30 base time
        let performance_factor = 1.0 + (1.0 - perf) * 0.15; // Performance adjustment
        let lap_time = base_time.mul_f64(performance_factor);
        
        // Check if this is the fastest lap
        if let Some((_, current_fastest)) = fastest_lap {
            if lap_time < *current_fastest {
                *fastest_lap = Some((driver_idx, lap_time));
            }
        } else {
            *fastest_lap = Some((driver_idx, lap_time));
        }
    }
}

// Display a summary of the current lap (top positions and gaps)
fn display_lap_summary(
    drivers: &[Driver],
    positions: &Vec<(usize, f64, Duration, bool)>,
    lap: u32,
    dnf_drivers: &Vec<usize>,
    fastest_lap: Option<(usize, Duration)>
) {
    // Skip unused variable warnings by using underscore prefix
    let _lap = lap;
    let _dnf_drivers = dnf_drivers; // Not directly used in this function
    
    // Show top 5 positions
    let max_to_show = 5.min(positions.len());
    let mut prev_gap: Option<Duration> = None;
    
    for i in 0..max_to_show {
        let (driver_idx, _, _, active) = positions[i];
        if !active {
            continue;
        }
        
        let driver = &drivers[driver_idx];
        let pos_str = format!("P{}", i+1);
        let pos_colored = match i {
            0 => pos_str.bright_yellow(),
            1 => pos_str.bright_white(),
            2 => pos_str.yellow(),
            _ => pos_str.normal(),
        };
        
        // Calculate gap to leader or car ahead
        let gap_str = if i == 0 {
            "Leader".to_string()
        } else {
            // We don't actually need to use prev_gap_time, just need to check if it exists
            let _prev_gap_exists = prev_gap.is_some();
            let gap_to_next = Duration::from_millis(
                (positions[i].1 - positions[i-1].1).abs() as u64 * 1000
            );
            format!("+{:.1}s", gap_to_next.as_secs_f64())
        };
        
        // Update gap for next iteration
        prev_gap = Some(Duration::from_secs_f64(positions[i].1 as f64));
        
        // Show fastest lap indicator
        let fl_indicator = if let Some((fl_driver, _)) = fastest_lap {
            if fl_driver == driver_idx {
                " 🟣".purple()
            } else {
                "".normal()
            }
        } else {
            "".normal()
        };
        
        println!("{:<4} {:<20} {:<15} {:<8} {}", 
            pos_colored,
            driver.name,
            driver.team.bright_cyan(),
            gap_str,
            fl_indicator
        );
    }
}

// Display the final race results
fn display_final_results(
    drivers: &[Driver], 
    positions: &Vec<(usize, f64, Duration, bool)>,
    dnf_drivers: &Vec<usize>,
    fastest_lap: Option<(usize, Duration)>
) {
    println!("\n{}", "RACE RESULTS".green().bold());
    println!("{}", "-".repeat(60));
    
    println!("{:<3} {:<20} {:<15} {:<10} {}", 
        "Pos".bold(),
        "Driver".bold(),
        "Team".bold(),
        "Time".bold(),
        "Points".bold()
    );
    
    println!("{}", "-".repeat(60));
    
    // Display finishers
    for (i, &(driver_idx, _, _, active)) in positions.iter().enumerate() {
        if !active {
            continue; // Skip DNFs for now
        }
        
        let driver = &drivers[driver_idx];
        
        let pos = i + 1;
        let pos_str = pos.to_string();
        let pos_colored = match pos {
            1 => pos_str.bright_yellow(),
            2 => pos_str.bright_white(),
            3 => pos_str.yellow(),
            _ => pos_str.normal(),
        };
        
        // Calculate points
        let mut points = match pos {
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
        
        // Add point for fastest lap if in top 10
        if let Some((fl_driver_idx, _)) = fastest_lap {
            if fl_driver_idx == driver_idx && pos <= 10 {
                points += 1;
            }
        }
        
        // Show fastest lap indicator
        let fl_indicator = if let Some((fl_driver, _)) = fastest_lap {
            if fl_driver == driver_idx {
                " 🟣 FASTEST LAP".purple()
            } else {
                "".normal()
            }
        } else {
            "".normal()
        };
        
        println!("{:<3} {:<20} {:<15} {:<10} {:<3}{}", 
            pos_colored,
            driver.name,
            driver.team,
            format!("+{:.3}s", (i as f64) * 2.5), // Simplified time gaps
            points,
            fl_indicator
        );
    }
    
    // Display DNFs
    for &driver_idx in dnf_drivers {
        let driver = &drivers[driver_idx];
        println!("{:<3} {:<20} {:<15} {:<10} {}", 
            "DNF".red(),
            driver.name,
            driver.team,
            "DNF".red(),
            0
        );
    }
    
    // Show fastest lap details
    if let Some((fl_driver_idx, fl_time)) = fastest_lap {
        let fl_driver = &drivers[fl_driver_idx];
        println!("\n{} {} - {} - {:.3}s", 
            "FASTEST LAP:".purple().bold(),
            fl_driver.name,
            fl_driver.team,
            fl_time.as_secs_f64()
        );
    }
}

// Helper function to multiply Duration by float
pub trait DurationExt {
    fn mul_f64(&self, factor: f64) -> Duration;
}

impl DurationExt for Duration {
    fn mul_f64(&self, factor: f64) -> Duration {
        let nanos = self.as_nanos() as f64 * factor;
        Duration::from_nanos(nanos as u64)
    }
}

// Wait for user to press Enter
fn wait_for_user_input() {
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
}