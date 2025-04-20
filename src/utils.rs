use colored::*;
use std::time::Duration;
use rand::Rng;
use crate::models::{Driver, RaceResult, QualifyingResult};

/// Convert a lap time string (e.g. "1:30.123") to Duration
#[allow(dead_code)]
pub fn parse_lap_time(time_str: &str) -> Option<Duration> {
    let parts: Vec<&str> = time_str.split(':').collect();
    
    match parts.len() {
        // Format: "1:30.123"
        2 => {
            let minutes: u64 = parts[0].parse().ok()?;
            let seconds_parts: Vec<&str> = parts[1].split('.').collect();
            if seconds_parts.len() != 2 {
                return None;
            }
            
            let seconds: u64 = seconds_parts[0].parse().ok()?;
            let milliseconds: u64 = seconds_parts[1].parse().ok()?;
            
            Some(Duration::from_millis(
                minutes * 60 * 1000 + seconds * 1000 + milliseconds
            ))
        },
        // Format: "30.123"
        1 => {
            let seconds_parts: Vec<&str> = parts[0].split('.').collect();
            if seconds_parts.len() != 2 {
                return None;
            }
            
            let seconds: u64 = seconds_parts[0].parse().ok()?;
            let milliseconds: u64 = seconds_parts[1].parse().ok()?;
            
            Some(Duration::from_millis(seconds * 1000 + milliseconds))
        },
        _ => None,
    }
}

/// Convert Duration to a formatted lap time string
#[allow(dead_code)]
pub fn format_duration_as_lap_time(duration: Duration) -> String {
    let total_millis = duration.as_millis();
    let minutes = total_millis / (60 * 1000);
    let seconds = (total_millis / 1000) % 60;
    let millis = total_millis % 1000;
    
    if minutes > 0 {
        format!("{}:{:02}.{:03}", minutes, seconds, millis)
    } else {
        format!("{}.{:03}", seconds, millis)
    }
}

/// Add random variation to a lap time
#[allow(dead_code)]
pub fn add_time_variation(base_time: Duration, variation_percent: f64) -> Duration {
    let mut rng = rand::thread_rng();
    let variation_factor = 1.0 + (rng.gen::<f64>() * 2.0 - 1.0) * variation_percent;
    let millis = (base_time.as_millis() as f64 * variation_factor) as u64;
    Duration::from_millis(millis)
}

/// Format race results in a nice table for terminal output
pub fn format_race_results(results: &[RaceResult]) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("{:<3} {:<20} {:<15} {:<10} {}\n", 
        "Pos".bold(), 
        "Driver".bold(), 
        "Team".bold(), 
        "Time".bold(),
        "Points".bold()
    ));
    
    output.push_str(&format!("{}\n", "-".repeat(60)));
    
    for result in results {
        let position = format!("{}", result.position);
        let position_colored = match result.position {
            1 => position.bright_yellow(),
            2 => position.bright_white(),
            3 => position.yellow(),
            _ => position.normal(),
        };
        
        let time_str = match &result.time {
            Some(time) => time.to_string(),
            None => result.status.clone(),
        };
        
        let team_color = get_team_color(&result.driver.team);
        let colored_team = match team_color {
            Color::BrightCyan => result.driver.team.bright_cyan(),
            Color::Blue => result.driver.team.blue(),
            Color::Red => result.driver.team.red(),
            Color::BrightYellow => result.driver.team.bright_yellow(),
            Color::Green => result.driver.team.green(),
            Color::Magenta => result.driver.team.magenta(),
            Color::BrightBlue => result.driver.team.bright_blue(),
            Color::White => result.driver.team.white(),
            Color::BrightRed => result.driver.team.bright_red(),
            _ => result.driver.team.normal(),
        };
        
        output.push_str(&format!("{:<3} {:<20} {:<15} {:<10} {}\n",
            position_colored,
            result.driver.name,
            colored_team,
            time_str,
            result.points
        ));
    }
    
    output
}

/// Format qualifying results in a nice table for terminal output
pub fn format_qualifying_results(results: &[QualifyingResult]) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("{:<3} {:<20} {:<15} {:<10} {:<10} {}\n", 
        "Pos".bold(), 
        "Driver".bold(), 
        "Team".bold(), 
        "Q1".bold(),
        "Q2".bold(),
        "Q3".bold()
    ));
    
    output.push_str(&format!("{}\n", "-".repeat(70)));
    
    for result in results {
        let position = format!("{}", result.position);
        let position_colored = match result.position {
            1 => position.bright_yellow(),
            2 => position.bright_white(),
            3 => position.yellow(),
            _ => position.normal(),
        };
        
        let team_color = get_team_color(&result.driver.team);
        let colored_team = match team_color {
            Color::BrightCyan => result.driver.team.bright_cyan(),
            Color::Blue => result.driver.team.blue(),
            Color::Red => result.driver.team.red(),
            Color::BrightYellow => result.driver.team.bright_yellow(),
            Color::Green => result.driver.team.green(),
            Color::Magenta => result.driver.team.magenta(),
            Color::BrightBlue => result.driver.team.bright_blue(),
            Color::White => result.driver.team.white(),
            Color::BrightRed => result.driver.team.bright_red(),
            _ => result.driver.team.normal(),
        };
        
        output.push_str(&format!("{:<3} {:<20} {:<15} {:<10} {:<10} {}\n",
            position_colored,
            result.driver.name,
            colored_team,
            result.q1.as_deref().unwrap_or("-"),
            result.q2.as_deref().unwrap_or("-"),
            result.q3.as_deref().unwrap_or("-")
        ));
    }
    
    output
}

/// Helper function to get color for F1 team
fn get_team_color(team: &str) -> Color {
    match team.to_lowercase().as_str() {
        team if team.contains("mercedes") => Color::BrightCyan,
        team if team.contains("red bull") => Color::Blue,
        team if team.contains("ferrari") => Color::Red,
        team if team.contains("mclaren") => Color::BrightYellow,
        team if team.contains("aston martin") => Color::Green,
        team if team.contains("alpine") => Color::Magenta,
        team if team.contains("williams") => Color::BrightBlue,
        team if team.contains("haas") => Color::White,
        team if team.contains("alfa") || team.contains("sauber") => Color::BrightRed,
        _ => Color::White,
    }
}

/// Generate a random mechanical failure based on driver reliability
pub fn simulate_mechanical_failure(driver: &Driver, reliability_factor: f64) -> bool {
    let mut rng = rand::thread_rng();
    
    // Base reliability varies by team (simplified model)
    let base_reliability = match driver.team.to_lowercase().as_str() {
        team if team.contains("mercedes") => 0.95,
        team if team.contains("red bull") => 0.96,
        team if team.contains("ferrari") => 0.94,
        team if team.contains("mclaren") => 0.95,
        team if team.contains("aston martin") => 0.93,
        team if team.contains("alpine") => 0.92,
        team if team.contains("williams") => 0.91,
        team if team.contains("haas") => 0.90,
        team if team.contains("alfa") || team.contains("sauber") => 0.90,
        _ => 0.92,
    };
    
    // Adjust with reliability factor
    let failure_chance = (1.0 - base_reliability) * (1.0 / reliability_factor);
    
    // Simulate failure
    rng.gen::<f64>() < failure_chance
}

/// Get random racing incident description
pub fn get_random_incident() -> &'static str {
    let incidents = [
        "Lost control in the corner",
        "Collision with another driver",
        "Puncture",
        "Engine failure",
        "Brake failure",
        "Hydraulic issue",
        "Electrical problem",
        "Gearbox failure",
        "Power unit issue",
        "Suspension damage",
        "Fuel pressure problem",
        "Cooling system issue",
    ];
    
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..incidents.len());
    incidents[index]
}

/// Convert GP name input to standardized format for API
pub fn normalize_gp_name(gp: &str) -> String {
    let normalized = gp.to_lowercase()
        .replace(" ", "-")
        .replace("grand prix", "")
        .replace("gp", "")
        .trim()
        .to_string();
        
    match normalized.as_str() {
        "monaco" => "monaco".to_string(),
        "monza" | "italian" => "monza".to_string(),
        "spa" | "belgian" => "spa".to_string(),
        "silverstone" | "british" => "silverstone".to_string(),
        "barcelona" | "spanish" | "spain" => "catalunya".to_string(),
        "melbourne" | "australia" | "australian" => "albert_park".to_string(),
        "montreal" | "canada" | "canadian" => "villeneuve".to_string(),
        "baku" | "azerbaijan" => "baku".to_string(),
        "hungaroring" | "hungary" | "hungarian" => "hungaroring".to_string(),
        "suzuka" | "japan" | "japanese" => "suzuka".to_string(),
        "singapore" => "marina_bay".to_string(),
        "austin" | "usa" | "us" => "americas".to_string(),
        "mexico" | "mexican" => "rodriguez".to_string(),
        "brazil" | "brazilian" | "interlagos" => "interlagos".to_string(),
        "abu-dhabi" | "abu dhabi" | "abudhabi" => "yas_marina".to_string(),
        "bahrain" => "bahrain".to_string(),
        "jeddah" | "saudi" | "saudi arabia" | "saudi-arabia" => "jeddah".to_string(),
        "imola" | "emilia romagna" => "imola".to_string(),
        "miami" => "miami".to_string(),
        "zandvoort" | "dutch" | "netherlands" => "zandvoort".to_string(),
        "las-vegas" | "las vegas" | "vegas" => "las_vegas".to_string(),
        "qatar" | "losail" => "losail".to_string(),
        _ => normalized,
    }
}