use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub id: String,
    pub code: String,
    pub name: String,
    pub team: String,
    pub number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    pub id: String,
    pub name: String,
    pub country: String,
    pub city: String,
    pub length_km: f64,
    pub laps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResult {
    pub position: u32,
    pub driver: Driver,
    pub time: Option<String>,
    pub points: u32,
    pub laps: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifyingResult {
    pub position: u32,
    pub driver: Driver,
    pub q1: Option<String>,
    pub q2: Option<String>,
    pub q3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeResult {
    pub position: u32,
    pub driver: Driver,
    pub time: Option<String>,
    pub laps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub season: u32,
    pub round: u32,
    pub name: String,
    pub circuit: Circuit,
    pub date: String,
    pub results: Vec<RaceResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParameters {
    pub reliability_factor: f64,
    pub weather_factor: f64,
    pub random_incidents: bool,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            reliability_factor: 0.95,
            weather_factor: 1.0,
            random_incidents: true,
        }
    }
}