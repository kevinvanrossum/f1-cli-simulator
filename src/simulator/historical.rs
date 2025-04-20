use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use crate::data::{DataInterface, DataManager};
use crate::models::{RaceResult, Circuit};
use crate::utils;

pub fn simulate(season: u32, gp: &str, session: &str, interactive: bool) -> Result<()> {
    let data_manager = DataManager;
    simulate_with_data_module(season, gp, session, interactive, &data_manager)
}

pub fn simulate_with_data_module(
    season: u32, 
    gp: &str, 
    session: &str,
    interactive: bool,
    data_module: &impl DataInterface
) -> Result<()> {
    println!("Loading historical data for {} GP {} - {} session", gp, season, session);
    
    match session.to_lowercase().as_str() {
        "race" => simulate_race(season, gp, interactive, data_module),
        "qualifying" => simulate_qualifying(season, gp, data_module),
        "practice" | "fp1" | "practice1" => simulate_practice(season, gp, 1, data_module),
        "fp2" | "practice2" => simulate_practice(season, gp, 2, data_module),
        "fp3" | "practice3" => simulate_practice(season, gp, 3, data_module),
        _ => Err(anyhow::anyhow!("Unknown session type: {}. Valid options are race, qualifying, practice, fp1, fp2, fp3", session)),
    }
}

fn simulate_race(season: u32, gp: &str, interactive: bool, data_module: &impl DataInterface) -> Result<()> {
    println!("{}", "Simulating historical race...".blue());
    
    let race = data_module.load_race_data(season, gp)?;
    
    // Display race information
    println!("\n{} - {}", race.name.bold(), race.date.italic());
    println!("{}, {}, {}", 
        race.circuit.name, 
        race.circuit.city,
        race.circuit.country
    );
    
    if interactive {
        simulate_interactive_historical_race(&race, &race.results)
    } else {
        // Display formatted results directly
        println!("\n{}", "Final Results:".green().bold());
        let formatted_results = utils::format_race_results(&race.results);
        println!("{}", formatted_results);
        Ok(())
    }
}

fn simulate_interactive_historical_race(race: &crate::models::Race, final_results: &[RaceResult]) -> Result<()> {
    println!("\n{}", "Interactive Historical Race Simulation".green().bold());
    println!("{}","-".repeat(50));
    
    // For historical races, we'll need to reconstruct a plausible race progression
    // based on the final results, as we don't have actual lap-by-lap data
    
    // Estimate total laps based on circuit
    let total_laps = estimate_laps_for_circuit(&race.circuit);
    
    // Create starting grid (often similar to final order but with some variations)
    let mut positions = create_starting_grid(final_results);
    
    println!("\n{}", "Starting Grid:".yellow());
    display_grid(&positions, final_results);
    
    println!("\n{}", "Press Enter to start the race...".green());
    wait_for_user_input();
    
    // Track DNFs - drivers who didn't finish the race
    let dnfs = identify_dnfs(final_results);
    let mut current_dnfs = Vec::new();
    
    // Track fastest lap
    let fastest_lap_driver = identify_fastest_lap(final_results);
    
    // Lap by lap simulation
    for lap in 1..=total_laps {
        println!("\n{}", format!("Lap {}/{}", lap, total_laps).bold());
        
        // Gradually move drivers toward their final positions
        update_positions_for_lap(&mut positions, final_results, lap, total_laps);
        
        // Check for DNFs that might happen on this lap
        if !dnfs.is_empty() {
            let lap_dnfs = check_for_lap_dnfs(&dnfs, lap, total_laps);
            for dnf in lap_dnfs {
                current_dnfs.push(dnf);
                println!("{}", format!("LAP {} - INCIDENT: {} - {}", 
                    lap, 
                    get_driver_name(final_results, dnf),
                    random_incident_for_driver(dnf)
                ).red());
            }
        }
        
        // Display current positions and status
        display_lap_status(&positions, final_results, lap, &current_dnfs, fastest_lap_driver);
        
        if lap < total_laps {
            // Interactive mode - wait for user to continue or auto-continue
            if lap % 10 == 0 || lap == total_laps - 1 {
                println!("\nPress Enter to continue...");
                wait_for_user_input();
            } else {
                // Short delay between laps for race feel
                thread::sleep(Duration::from_millis(800));
            }
        }
    }
    
    // Display final results
    println!("\n{}", "RACE COMPLETE".green().bold());
    println!("{}", "Final Results:".green().bold());
    let formatted_results = utils::format_race_results(final_results);
    println!("{}", formatted_results);
    
    Ok(())
}

// Estimate laps for a given circuit based on available data or defaults
fn estimate_laps_for_circuit(circuit: &Circuit) -> u32 {
    let circuit_laps: HashMap<&str, u32> = [
        ("monza", 53),
        ("monaco", 78),
        ("spa", 44),
        ("silverstone", 52),
        ("bahrain", 57),
        ("jeddah", 50),
        ("albert_park", 58),
        ("baku", 51),
        ("miami", 57),
        ("imola", 63),
        ("monaco", 78),
        ("catalunya", 66),
        ("villeneuve", 70),
        ("red_bull_ring", 71),
        ("silverstone", 52),
        ("hungaroring", 70),
        ("spa", 44),
        ("zandvoort", 72),
        ("monza", 53),
        ("marina_bay", 62),
        ("suzuka", 53),
        ("losail", 57),
        ("americas", 56),
        ("rodriguez", 71),
        ("interlagos", 71),
        ("vegas", 50),
        ("yas_marina", 58),
    ].iter().cloned().collect();
    
    *circuit_laps.get(circuit.id.as_str()).unwrap_or(&(circuit.laps.max(50)))
}

// Create a plausible starting grid based on final results
fn create_starting_grid(final_results: &[RaceResult]) -> Vec<usize> {
    let mut grid: Vec<usize> = (0..final_results.len()).collect();
    
    // Adjust the grid to be somewhat similar to the final results
    // but with some realistic changes - especially for the mid-field
    for i in 0..grid.len() {
        // Front-runners often start near the front
        if i < 3 && grid[i] < 6 {
            // Keep top drivers near the front
            continue;
        }
        
        // Mid-field can have more variance
        if i >= 3 && i < grid.len() - 3 {
            // Allow for some position swaps in the midfield
            if rand::random::<f32>() < 0.4 {
                let swap_pos = (i as i32 + if rand::random() { 1 } else { -1 }).max(3).min((grid.len() - 4) as i32) as usize;
                grid.swap(i, swap_pos);
            }
        }
    }
    
    grid
}

// Display the current grid
fn display_grid(positions: &[usize], results: &[RaceResult]) {
    for (pos, &driver_idx) in positions.iter().enumerate() {
        if driver_idx < results.len() {
            let driver = &results[driver_idx].driver;
            println!("{:2}. {} - {}", pos + 1, driver.code, driver.team);
        }
    }
}

// Identify drivers who didn't finish the race
fn identify_dnfs(results: &[RaceResult]) -> Vec<usize> {
    let mut dnfs = Vec::new();
    
    for (idx, result) in results.iter().enumerate() {
        if !result.status.contains("Finished") && !result.status.contains("+") {
            dnfs.push(idx);
        }
    }
    
    dnfs
}

// Identify the driver with the fastest lap
fn identify_fastest_lap(results: &[RaceResult]) -> Option<usize> {
    // In real data, this would be marked specifically
    // For now, let's just assume one of the top 5 had the fastest lap
    if !results.is_empty() {
        let top_pos = results.len().min(5);
        Some(rand::random::<usize>() % top_pos)
    } else {
        None
    }
}

// Update positions gradually over the race to match final results
fn update_positions_for_lap(positions: &mut Vec<usize>, final_results: &[RaceResult], current_lap: u32, total_laps: u32) {
    // Calculate how close we are to the end of the race
    let race_progress = current_lap as f32 / total_laps as f32;
    
    // Determine overtaking probability based on race progress
    // More likely in early and mid-race, less likely near the end
    let overtake_probability = match race_progress {
        p if p < 0.1 => 0.3,  // First 10% of race - lots of position changes
        p if p < 0.7 => 0.15, // Mid-race - moderate changes
        p if p < 0.9 => 0.1,  // Late race - fewer changes
        _ => 0.05,            // Final laps - minimal changes
    };
    
    // Create a target position ordering based on final results
    let target: Vec<usize> = (0..final_results.len()).collect();
    
    // For each position, consider if we need to make an overtake to move toward final order
    for i in 0..positions.len() - 1 {
        // Find where current driver should be in final results
        let current_driver = positions[i];
        let next_driver = positions[i + 1];
        
        let current_target_pos = target.iter().position(|&x| x == current_driver).unwrap_or(i);
        let next_target_pos = target.iter().position(|&x| x == next_driver).unwrap_or(i + 1);
        
        // If the next driver should be ahead of current driver in final results,
        // consider an overtake with some probability
        if next_target_pos < current_target_pos && rand::random::<f32>() < overtake_probability {
            positions.swap(i, i + 1);
        }
    }
}

// Check which DNFs should happen on the current lap
fn check_for_lap_dnfs(all_dnfs: &[usize], current_lap: u32, total_laps: u32) -> Vec<usize> {
    let mut lap_dnfs = Vec::new();
    
    for &dnf_idx in all_dnfs {
        // Distribute DNFs throughout the race, but more likely in the middle
        // First few laps and last few laps typically have fewer DNFs
        let dnf_probability = match current_lap as f32 / total_laps as f32 {
            p if p < 0.1 => 0.01,           // First 10% - few DNFs
            p if p < 0.3 => 0.03,           // Early race
            p if p < 0.7 => 0.04,           // Mid race - most DNFs happen here
            p if p < 0.9 => 0.02,           // Late race
            _ => 0.01,                      // Final laps - few DNFs
        };
        
        if rand::random::<f32>() < dnf_probability {
            lap_dnfs.push(dnf_idx);
        }
    }
    
    lap_dnfs
}

// Get a driver's name from their result index
fn get_driver_name(results: &[RaceResult], idx: usize) -> String {
    if idx < results.len() {
        results[idx].driver.name.clone()
    } else {
        "Unknown Driver".to_string()
    }
}

// Generate a random plausible incident for a driver DNF
fn random_incident_for_driver(driver_idx: usize) -> String {
    let incidents = [
        "Engine failure",
        "Hydraulics issue",
        "Gearbox failure",
        "Collision damage",
        "Brake failure",
        "Power unit issue",
        "Mechanical failure",
        "Oil pressure drop",
        "Electrical issues",
        "Suspension damage",
        "Tire puncture",
        "Overheating",
    ];
    
    // Use driver index to influence incident type slightly, but still with randomness
    let incident_idx = (driver_idx + (rand::random::<usize>() % 5)) % incidents.len();
    incidents[incident_idx].to_string()
}

// Display current race status for a lap
fn display_lap_status(
    positions: &[usize], 
    results: &[RaceResult], 
    lap: u32,
    dnfs: &[usize],
    fastest_lap: Option<usize>
) {
    // Show top positions (limited to what's visible on screen)
    let max_to_show = 10.min(positions.len());
    
    for i in 0..max_to_show {
        let driver_idx = positions[i];
        if dnfs.contains(&driver_idx) {
            continue; // Skip DNF'd drivers
        }
        
        if driver_idx < results.len() {
            let driver = &results[driver_idx].driver;
            let pos_str = format!("P{}", i+1);
            let pos_colored = match i {
                0 => pos_str.bright_yellow(),
                1 => pos_str.bright_white(),
                2 => pos_str.yellow(),
                _ => pos_str.normal(),
            };
            
            // Gap calculation would be more complex in reality
            // Here we'll just show a simplified version
            let gap_str = if i == 0 {
                "Leader".to_string()
            } else {
                format!("+{:.1}s", (i as f32) * 0.8 + (rand::random::<f32>() * 0.4))
            };
            
            // Show fastest lap indicator
            let fl_indicator = if let Some(fl_idx) = fastest_lap {
                if fl_idx == driver_idx && lap > 5 {  // Show fastest lap after a few laps
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
}

fn simulate_qualifying(season: u32, gp: &str, data_module: &impl DataInterface) -> Result<()> {
    println!("{}", "Simulating historical qualifying session...".blue());
    
    match data_module.load_qualifying_data(season, gp) {
        Ok(results) => {
            let formatted_results = utils::format_qualifying_results(&results);
            println!("{}", formatted_results);
            Ok(())
        },
        Err(e) => {
            println!("{}", "Qualifying data is not yet implemented.".yellow());
            println!("{}", "This feature will be available in a future update.".yellow());
            Err(e)
        }
    }
}

fn simulate_practice(season: u32, gp: &str, practice_number: u32, data_module: &impl DataInterface) -> Result<()> {
    println!("{}", format!("Simulating historical FP{} session...", practice_number).blue());
    
    match data_module.load_practice_data(season, gp, practice_number) {
        Ok(_) => {
            // Display practice results (would need a format function for this)
            println!("Practice results loaded successfully");
            Ok(())
        },
        Err(e) => {
            println!("{}", "Practice session data is not yet implemented.".yellow());
            println!("{}", "This feature will be available in a future update.".yellow());
            Err(e)
        }
    }
}

// Wait for user to press Enter
fn wait_for_user_input() {
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
}