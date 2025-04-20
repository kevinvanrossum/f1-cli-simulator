use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use serde_json::Value;
use crate::models::{Driver, Circuit, Race, RaceResult, QualifyingResult, PracticeResult};
use crate::utils::normalize_gp_name;

const API_BASE_URL: &str = "https://ergast.com/api/f1";
const DATA_DIR: &str = "./data";
const CURRENT_SEASON: u32 = 2025;

/// Data interface trait for dependency injection and testing
pub trait DataInterface {
    fn load_race_data(&self, season: u32, gp: &str) -> Result<Race>;
    fn load_qualifying_data(&self, season: u32, gp: &str) -> Result<Vec<QualifyingResult>>;
    fn load_practice_data(&self, season: u32, gp: &str, practice_number: u32) -> Result<Vec<PracticeResult>>;
}

/// Default implementation that uses the file system and API
pub struct DataManager;

impl DataInterface for DataManager {
    fn load_race_data(&self, season: u32, gp: &str) -> Result<Race> {
        load_race_data(season, gp)
    }

    fn load_qualifying_data(&self, season: u32, gp: &str) -> Result<Vec<QualifyingResult>> {
        load_qualifying_data(season, gp)
    }

    fn load_practice_data(&self, season: u32, gp: &str, practice_number: u32) -> Result<Vec<PracticeResult>> {
        load_practice_data(season, gp, practice_number)
    }
}

/// Initialize data directory if it doesn't exist
fn ensure_data_dir() -> Result<()> {
    let path = Path::new(DATA_DIR);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get the file path for a season's data
fn get_season_data_path(season: u32) -> String {
    format!("{}/season_{}.json", DATA_DIR, season)
}

/// Get the file path for a specific race's data
fn get_race_data_path(season: u32, gp_name: &str) -> String {
    format!("{}/race_{}_{}.json", DATA_DIR, season, gp_name)
}

/// Get the file path for qualifying data
fn get_qualifying_data_path(season: u32, gp_name: &str) -> String {
    format!("{}/qualifying_{}_{}.json", DATA_DIR, season, gp_name)
}

/// Get the file path for practice data
fn get_practice_data_path(season: u32, gp_name: &str, practice_number: u32) -> String {
    format!("{}/practice{}_{}_{}.json", DATA_DIR, practice_number, season, gp_name)
}

/// List available race data
pub fn list_available_data(filter_season: Option<u32>) -> Result<()> {
    ensure_data_dir()?;
    
    let data_dir = Path::new(DATA_DIR);
    
    // Check if data directory exists
    if !data_dir.exists() {
        println!("{}", "No data available. Run 'update' command to fetch race data.".red());
        return Ok(());
    }
    
    let mut has_data = false;
    let mut seasons: HashMap<u32, Vec<String>> = HashMap::new();
    
    // Go through data directory and catalog files
    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().unwrap_or_default();
        
        // Season data files
        if file_name.starts_with("season_") && file_name.ends_with(".json") {
            let season: u32 = file_name
                .replace("season_", "")
                .replace(".json", "")
                .parse()
                .unwrap_or(0);
                
            if season > 0 && (filter_season.is_none() || filter_season == Some(season)) {
                seasons.entry(season).or_insert_with(Vec::new);
                has_data = true;
            }
        }
        
        // Race data files
        else if file_name.starts_with("race_") && file_name.ends_with(".json") {
            let file_name_string = file_name
                .replace("race_", "")
                .replace(".json", "");
            let parts: Vec<&str> = file_name_string
                .split('_')
                .collect();
                
            if parts.len() >= 2 {
                if let Ok(season) = parts[0].parse::<u32>() {
                    if filter_season.is_none() || filter_season == Some(season) {
                        let gp = parts[1..].join("_");
                        seasons.entry(season).or_insert_with(Vec::new).push(gp);
                        has_data = true;
                    }
                }
            }
        }
    }
    
    if !has_data {
        if let Some(year) = filter_season {
            println!("{}", format!("No data available for season {}. Run 'update' command to fetch race data.", year).yellow());
        } else {
            println!("{}", "No data available. Run 'update' command to fetch race data.".yellow());
        }
        return Ok(());
    }
    
    // Print found data
    for (season, gp_list) in seasons.iter().filter(|(s, _)| filter_season.is_none() || filter_season == Some(**s)) {
        println!("\n{} {}", "Season".green(), season.to_string().green().bold());
        println!("{}", "-".repeat(40));
        
        if gp_list.is_empty() {
            println!("  {}", "Season data available, no specific races downloaded".italic());
        } else {
            for gp in gp_list {
                println!("  • {}", gp.replace("_", " ").to_uppercase());
            }
        }
    }
    
    Ok(())
}

/// Update F1 race data from the Ergast API
pub fn update_data() -> Result<()> {
    ensure_data_dir()?;
    
    let client = Client::new();
    
    println!("{}", "Updating F1 race data...".green());
    
    // Fetch data for last few seasons and current season
    let seasons_to_fetch = vec![CURRENT_SEASON - 2, CURRENT_SEASON - 1, CURRENT_SEASON];
    
    for season in seasons_to_fetch {
        println!("\n{} {}", "Fetching data for season".blue(), season.to_string().blue().bold());
        
        // Create a progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.set_message(format!("Fetching season {} schedule...", season));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        
        // Fetch season schedule
        let season_url = format!("{}/{}/circuits.json", API_BASE_URL, season);
        let season_response = client.get(&season_url).send()
            .with_context(|| format!("Failed to fetch season {} data", season))?;
            
        if !season_response.status().is_success() {
            pb.finish_with_message(format!("Season {} data not available (status: {})", season, season_response.status()));
            continue;
        }
        
        let season_data: Value = season_response.json()?;
        
        // Extract circuit data
        let circuits = match season_data.get("MRData")
            .and_then(|d| d.get("CircuitTable"))
            .and_then(|t| t.get("Circuits"))
        {
            Some(circuits) => circuits,
            None => {
                pb.finish_with_message(format!("No circuit data found for season {}", season));
                continue;
            }
        };
        
        // Save season data
        let season_path = get_season_data_path(season);
        fs::write(&season_path, serde_json::to_string_pretty(&circuits)?)?;
        pb.finish_with_message(format!("Saved season {} data", season));
        
        // Fetch data for each race
        if let Some(circuits_array) = circuits.as_array() {
            for circuit in circuits_array {
                if let Some(circuit_id) = circuit.get("circuitId").and_then(|id| id.as_str()) {
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::default_spinner()
                            .template("{spinner:.green} {msg}")
                            .unwrap()
                    );
                    pb.set_message(format!("Fetching data for {} GP...", circuit_id));
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));
                    
                    // Fetch race results
                    let race_url = format!("{}/{}/circuits/{}/results.json", API_BASE_URL, season, circuit_id);
                    let race_response = client.get(&race_url).send();
                    
                    match race_response {
                        Ok(response) if response.status().is_success() => {
                            let race_data: Value = response.json()?;
                            if let Some(races) = race_data.get("MRData")
                                .and_then(|d| d.get("RaceTable"))
                                .and_then(|t| t.get("Races"))
                            {
                                let race_path = get_race_data_path(season, circuit_id);
                                fs::write(&race_path, serde_json::to_string_pretty(&races)?)?;
                                pb.finish_with_message(format!("Saved data for {} GP", circuit_id));
                            } else {
                                pb.finish_with_message(format!("No race data found for {} GP", circuit_id));
                            }
                        },
                        _ => pb.finish_with_message(format!("Failed to fetch data for {} GP", circuit_id)),
                    }
                }
            }
        }
    }
    
    println!("\n{}", "F1 race data update completed".green().bold());
    Ok(())
}

/// Fetch data for a specific race from the Ergast API
fn fetch_race_data(client: &Client, season: u32, gp: &str) -> Result<()> {
    println!("{}", format!("Race data for {} GP {} not found locally, fetching from API...", gp, season).yellow());
    
    // Create a progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message(format!("Fetching data for {} GP {}...", gp, season));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    // First, we need to determine the correct circuit ID
    let circuit_id = normalize_gp_name(gp);
    
    // Fetch race results
    let race_url = format!("{}/{}/circuits/{}/results.json", API_BASE_URL, season, circuit_id);
    let race_response = client.get(&race_url).send();
    
    match race_response {
        Ok(response) if response.status().is_success() => {
            let race_data: Value = response.json()?;
            if let Some(races) = race_data.get("MRData")
                .and_then(|d| d.get("RaceTable"))
                .and_then(|t| t.get("Races"))
            {
                let race_path = get_race_data_path(season, &circuit_id);
                fs::write(&race_path, serde_json::to_string_pretty(&races)?)?;
                pb.finish_with_message(format!("Successfully fetched data for {} GP {}", gp, season));
                Ok(())
            } else {
                pb.finish_with_message(format!("No race data found for {} GP {}", gp, season));
                Err(anyhow::anyhow!("No race data found for {} GP {}", gp, season))
            }
        },
        Ok(response) => {
            pb.finish_with_message(format!("Failed to fetch data for {} GP {} (status: {})", gp, season, response.status()));
            Err(anyhow::anyhow!("API returned error status: {}", response.status()))
        },
        Err(e) => {
            pb.finish_with_message(format!("Failed to connect to API for {} GP {}", gp, season));
            Err(anyhow::anyhow!("Failed to connect to API: {}", e))
        }
    }
}

/// Fetch qualifying data for a specific race from the Ergast API
fn fetch_qualifying_data(client: &Client, season: u32, gp: &str) -> Result<()> {
    println!("{}", format!("Qualifying data for {} GP {} not found locally, fetching from API...", gp, season).yellow());
    
    // Create a progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message(format!("Fetching qualifying data for {} GP {}...", gp, season));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    // First, we need to determine the correct circuit ID
    let circuit_id = normalize_gp_name(gp);
    
    // Fetch qualifying results
    let qualifying_url = format!("{}/{}/circuits/{}/qualifying.json", API_BASE_URL, season, circuit_id);
    let qualifying_response = client.get(&qualifying_url).send();
    
    match qualifying_response {
        Ok(response) if response.status().is_success() => {
            let qualifying_data: Value = response.json()?;
            if let Some(races) = qualifying_data.get("MRData")
                .and_then(|d| d.get("RaceTable"))
                .and_then(|t| t.get("Races"))
            {
                let qualifying_path = get_qualifying_data_path(season, &circuit_id);
                fs::write(&qualifying_path, serde_json::to_string_pretty(&races)?)?;
                pb.finish_with_message(format!("Successfully fetched qualifying data for {} GP {}", gp, season));
                Ok(())
            } else {
                pb.finish_with_message(format!("No qualifying data found for {} GP {}", gp, season));
                Err(anyhow::anyhow!("No qualifying data found for {} GP {}", gp, season))
            }
        },
        Ok(response) => {
            pb.finish_with_message(format!("Failed to fetch qualifying data for {} GP {} (status: {})", gp, season, response.status()));
            Err(anyhow::anyhow!("API returned error status: {}", response.status()))
        },
        Err(e) => {
            pb.finish_with_message(format!("Failed to connect to API for {} GP {} qualifying", gp, season));
            Err(anyhow::anyhow!("Failed to connect to API: {}", e))
        }
    }
}

/// Fetch practice data for a specific race from the Ergast API
fn fetch_practice_data(client: &Client, season: u32, gp: &str, practice_number: u32) -> Result<()> {
    println!("{}", format!("Practice data for {} GP {} FP{} not found locally, fetching from API...", gp, season, practice_number).yellow());
    
    // Create a progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message(format!("Fetching FP{} data for {} GP {}...", practice_number, gp, season));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    // First, we need to determine the correct circuit ID
    let circuit_id = normalize_gp_name(gp);
    
    // Determine the practice session from the number
    let session = match practice_number {
        1 => "fp1",
        2 => "fp2",
        3 => "fp3",
        _ => return Err(anyhow::anyhow!("Invalid practice session number: {}", practice_number)),
    };
    
    // Fetch practice results
    let practice_url = format!("{}/{}/circuits/{}/{}/results.json", API_BASE_URL, season, circuit_id, session);
    let practice_response = client.get(&practice_url).send();
    
    match practice_response {
        Ok(response) if response.status().is_success() => {
            let practice_data: Value = response.json()?;
            if let Some(races) = practice_data.get("MRData")
                .and_then(|d| d.get("RaceTable"))
                .and_then(|t| t.get("Races"))
            {
                let practice_path = get_practice_data_path(season, &circuit_id, practice_number);
                fs::write(&practice_path, serde_json::to_string_pretty(&races)?)?;
                pb.finish_with_message(format!("Successfully fetched FP{} data for {} GP {}", practice_number, gp, season));
                Ok(())
            } else {
                pb.finish_with_message(format!("No FP{} data found for {} GP {}", practice_number, gp, season));
                Err(anyhow::anyhow!("No FP{} data found for {} GP {}", practice_number, gp, season))
            }
        },
        Ok(response) => {
            pb.finish_with_message(format!("Failed to fetch FP{} data for {} GP {} (status: {})", practice_number, gp, season, response.status()));
            Err(anyhow::anyhow!("API returned error status: {}", response.status()))
        },
        Err(e) => {
            pb.finish_with_message(format!("Failed to connect to API for {} GP {} FP{}", gp, season, practice_number));
            Err(anyhow::anyhow!("Failed to connect to API: {}", e))
        }
    }
}

/// Load race data for a specific GP
pub fn load_race_data(season: u32, gp: &str) -> Result<Race> {
    ensure_data_dir()?;
    let normalized_gp = normalize_gp_name(gp);
    let file_path = get_race_data_path(season, &normalized_gp);
    
    // If the file doesn't exist, attempt to fetch it
    if !Path::new(&file_path).exists() {
        let client = Client::new();
        fetch_race_data(&client, season, gp)?;
    }
    
    // Now try to load the data (which should exist now if the fetch was successful)
    if !Path::new(&file_path).exists() {
        return Err(anyhow::anyhow!(
            "Unable to retrieve race data for {} GP {}. Race may not exist or network issues occurred.",
            gp, season
        ));
    }
    
    let data = fs::read_to_string(&file_path)?;
    let race_data: Value = serde_json::from_str(&data)?;
    
    // Process the race data into our model
    if let Some(races) = race_data.as_array() {
        if let Some(race) = races.first() {
            let circuit = parse_circuit(race)?;
            let results = parse_results(race)?;
            
            let race_name = race.get("raceName")
                .and_then(|n| n.as_str())
                .unwrap_or(&normalized_gp)
                .to_string();
                
            let date = race.get("date")
                .and_then(|d| d.as_str())
                .unwrap_or("Unknown")
                .to_string();
                
            let round = race.get("round")
                .and_then(|r| r.as_str())
                .and_then(|r| r.parse::<u32>().ok())
                .unwrap_or(0);
                
            return Ok(Race {
                season,
                round,
                name: race_name,
                circuit,
                date,
                results,
            });
        }
    }
    
    Err(anyhow::anyhow!("Failed to parse race data"))
}

/// Parse circuit information from race data
fn parse_circuit(race: &Value) -> Result<Circuit> {
    if let Some(circuit_data) = race.get("Circuit") {
        let id = circuit_data.get("circuitId")
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string();
            
        let name = circuit_data.get("circuitName")
            .and_then(|name| name.as_str())
            .unwrap_or("Unknown Circuit")
            .to_string();
        
        let location = circuit_data.get("Location");
        
        let country = location.and_then(|l| l.get("country"))
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown")
            .to_string();
            
        let city = location.and_then(|l| l.get("locality"))
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown")
            .to_string();
            
        // These fields aren't in the API, so we'll use defaults
        let length_km = 5.0; // Default circuit length
        let laps = 50;      // Default number of laps
        
        return Ok(Circuit {
            id,
            name,
            country,
            city,
            length_km,
            laps,
        });
    }
    
    Err(anyhow::anyhow!("Failed to parse circuit data"))
}

/// Parse race results from race data
fn parse_results(race: &Value) -> Result<Vec<RaceResult>> {
    let mut results = Vec::new();
    
    if let Some(results_data) = race.get("Results").and_then(|r| r.as_array()) {
        for (index, result) in results_data.iter().enumerate() {
            let position = result.get("position")
                .and_then(|p| p.as_str())
                .and_then(|p| p.parse::<u32>().ok())
                .unwrap_or((index + 1) as u32);
                
            let driver = parse_driver(result)?;
            
            let time = result.get("Time")
                .and_then(|t| t.get("time"))
                .and_then(|t| t.as_str())
                .map(|t| t.to_string());
                
            let points = result.get("points")
                .and_then(|p| p.as_str())
                .and_then(|p| p.parse::<u32>().ok())
                .unwrap_or(0);
                
            let laps = result.get("laps")
                .and_then(|l| l.as_str())
                .and_then(|l| l.parse::<u32>().ok())
                .unwrap_or(0);
                
            let status = result.get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("Unknown")
                .to_string();
                
            results.push(RaceResult {
                position,
                driver,
                time,
                points,
                laps,
                status,
            });
        }
    }
    
    Ok(results)
}

/// Parse driver information from result data
fn parse_driver(result: &Value) -> Result<Driver> {
    if let Some(driver_data) = result.get("Driver") {
        let id = driver_data.get("driverId")
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string();
            
        let code = driver_data.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("???")
            .to_string();
            
        let first_name = driver_data.get("givenName")
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown")
            .to_string();
            
        let last_name = driver_data.get("familyName")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
            
        let name = format!("{} {}", first_name, last_name);
        
        let team = result.get("Constructor")
            .and_then(|c| c.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown Team")
            .to_string();
            
        let number = driver_data.get("permanentNumber")
            .and_then(|n| n.as_str())
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(0);
            
        return Ok(Driver {
            id,
            code,
            name,
            team,
            number,
        });
    }
    
    Err(anyhow::anyhow!("Failed to parse driver data"))
}

/// Load qualifying data for a specific GP
pub fn load_qualifying_data(season: u32, gp: &str) -> Result<Vec<QualifyingResult>> {
    ensure_data_dir()?;
    let normalized_gp = normalize_gp_name(gp);
    let file_path = get_qualifying_data_path(season, &normalized_gp);
    
    // If the file doesn't exist, attempt to fetch it
    if !Path::new(&file_path).exists() {
        let client = Client::new();
        fetch_qualifying_data(&client, season, gp)?;
    }
    
    // Now try to load the data (which should exist now if the fetch was successful)
    if !Path::new(&file_path).exists() {
        return Err(anyhow::anyhow!(
            "Unable to retrieve qualifying data for {} GP {}. Data may not exist or network issues occurred.",
            gp, season
        ));
    }
    
    let data = fs::read_to_string(&file_path)?;
    let qualifying_data: Value = serde_json::from_str(&data)?;
    
    // Process the qualifying data into our model
    let mut qualifying_results = Vec::new();
    
    if let Some(races) = qualifying_data.as_array() {
        if let Some(race) = races.first() {
            if let Some(qualifying_results_data) = race.get("QualifyingResults").and_then(|r| r.as_array()) {
                for (index, result) in qualifying_results_data.iter().enumerate() {
                    let position = result.get("position")
                        .and_then(|p| p.as_str())
                        .and_then(|p| p.parse::<u32>().ok())
                        .unwrap_or((index + 1) as u32);
                        
                    let driver = parse_driver(result)?;
                    
                    let q1_time = result.get("Q1")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_string());
                        
                    let q2_time = result.get("Q2")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_string());
                        
                    let q3_time = result.get("Q3")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_string());
                        
                    qualifying_results.push(QualifyingResult {
                        position,
                        driver,
                        q1: q1_time,
                        q2: q2_time,
                        q3: q3_time,
                    });
                }
            }
        }
    }
    
    if qualifying_results.is_empty() {
        return Err(anyhow::anyhow!("No qualifying results found for {} GP {}", gp, season));
    }
    
    Ok(qualifying_results)
}

/// Load practice data for a specific GP
pub fn load_practice_data(season: u32, gp: &str, practice_number: u32) -> Result<Vec<PracticeResult>> {
    ensure_data_dir()?;
    let normalized_gp = normalize_gp_name(gp);
    let file_path = get_practice_data_path(season, &normalized_gp, practice_number);
    
    // If the file doesn't exist, attempt to fetch it
    if !Path::new(&file_path).exists() {
        let client = Client::new();
        fetch_practice_data(&client, season, gp, practice_number)?;
    }
    
    // Now try to load the data (which should exist now if the fetch was successful)
    if !Path::new(&file_path).exists() {
        return Err(anyhow::anyhow!(
            "Unable to retrieve practice data for {} GP {} FP{}. Data may not exist or network issues occurred.",
            gp, season, practice_number
        ));
    }
    
    let data = fs::read_to_string(&file_path)?;
    let practice_data: Value = serde_json::from_str(&data)?;
    
    // Process the practice data into our model
    let mut practice_results = Vec::new();
    
    if let Some(races) = practice_data.as_array() {
        if let Some(race) = races.first() {
            if let Some(practice_results_data) = race.get("PracticeResults").and_then(|r| r.as_array()) {
                for (index, result) in practice_results_data.iter().enumerate() {
                    let position = result.get("position")
                        .and_then(|p| p.as_str())
                        .and_then(|p| p.parse::<u32>().ok())
                        .unwrap_or((index + 1) as u32);
                        
                    let driver = parse_driver(result)?;
                    
                    let time = result.get("time")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_string());

                    let laps = result.get("laps")
                        .and_then(|l| l.as_str())
                        .and_then(|l| l.parse::<u32>().ok())
                        .unwrap_or(0);
                        
                    practice_results.push(PracticeResult {
                        position,
                        driver,
                        time,
                        laps,
                    });
                }
            }
        }
    }
    
    if practice_results.is_empty() {
        return Err(anyhow::anyhow!("No practice results found for {} GP {} FP{}", gp, season, practice_number));
    }
    
    Ok(practice_results)
}

/// Get current season's driver standings
#[allow(dead_code)]
pub fn get_driver_standings(season: u32) -> Result<HashMap<String, u32>> {
    let mut standings = HashMap::new();
    
    // This would be implemented to aggregate points from all races
    // For now, just return dummy data
    if season == CURRENT_SEASON {
        standings.insert("Max Verstappen".to_string(), 230);
        standings.insert("Lando Norris".to_string(), 190);
        standings.insert("Charles Leclerc".to_string(), 186);
        standings.insert("Carlos Sainz".to_string(), 168);
        standings.insert("Lewis Hamilton".to_string(), 152);
    }
    
    Ok(standings)
}