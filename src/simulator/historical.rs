use anyhow::Result;
use colored::Colorize;

use crate::data::{DataInterface, DataManager};
use crate::utils;

pub fn simulate(season: u32, gp: &str, session: &str) -> Result<()> {
    let data_manager = DataManager;
    simulate_with_data_module(season, gp, session, &data_manager)
}

pub fn simulate_with_data_module(
    season: u32, 
    gp: &str, 
    session: &str, 
    data_module: &impl DataInterface
) -> Result<()> {
    println!("Loading historical data for {} GP {} - {} session", gp, season, session);
    
    match session.to_lowercase().as_str() {
        "race" => simulate_race(season, gp, data_module),
        "qualifying" => simulate_qualifying(season, gp, data_module),
        "practice" | "fp1" | "practice1" => simulate_practice(season, gp, 1, data_module),
        "fp2" | "practice2" => simulate_practice(season, gp, 2, data_module),
        "fp3" | "practice3" => simulate_practice(season, gp, 3, data_module),
        _ => Err(anyhow::anyhow!("Unknown session type: {}. Valid options are race, qualifying, practice, fp1, fp2, fp3", session)),
    }
}

fn simulate_race(season: u32, gp: &str, data_module: &impl DataInterface) -> Result<()> {
    println!("{}", "Simulating historical race...".blue());
    
    let race = data_module.load_race_data(season, gp)?;
    
    // Display race information
    println!("\n{} - {}", race.name.bold(), race.date.italic());
    println!("{}, {}, {}", 
        race.circuit.name, 
        race.circuit.city,
        race.circuit.country
    );
    println!("\n{}", "Final Results:".green().bold());
    
    // Display formatted results
    let formatted_results = utils::format_race_results(&race.results);
    println!("{}", formatted_results);
    
    Ok(())
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