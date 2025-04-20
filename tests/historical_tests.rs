//! Unit tests for historical race simulation functionality

use anyhow::Result;
use mockall::predicate::*;
use mockall::*;
use std::sync::{Arc, Mutex};
use std::io::Write;

// Import the crate modules - use the crate name with underscores instead of hyphens
use f1_cli_simulator::data::DataInterface;
use f1_cli_simulator::models::{Circuit, Driver, PracticeResult, QualifyingResult, Race, RaceResult};
use f1_cli_simulator::simulator::historical;

// Mocked data module to avoid real API calls during tests
mock! {
    pub DataModule {}

    impl DataInterface for DataModule {
        fn load_race_data(&self, season: u32, gp: &str) -> Result<Race>;
        fn load_qualifying_data(&self, season: u32, gp: &str) -> Result<Vec<QualifyingResult>>;
        fn load_practice_data(&self, season: u32, gp: &str, practice_number: u32) -> Result<Vec<PracticeResult>>;
    }
}

// Helper function to create a mock race for testing
fn create_mock_race(season: u32, gp: &str) -> Race {
    Race {
        season,
        round: 1,
        name: format!("{} Grand Prix", gp),
        circuit: Circuit {
            id: gp.to_string(),
            name: format!("{} Circuit", gp),
            country: "Test Country".to_string(),
            city: "Test City".to_string(),
            length_km: 5.0,
            laps: 50,
        },
        date: "2023-07-15".to_string(),
        results: vec![
            RaceResult {
                position: 1,
                driver: Driver {
                    id: "driver1".to_string(),
                    code: "DRV".to_string(),
                    name: "Test Driver".to_string(),
                    team: "Test Team".to_string(),
                    number: 1,
                },
                time: Some("1:30:45.123".to_string()),
                points: 25,
                laps: 50,
                status: "Finished".to_string(),
            },
        ],
    }
}

// Helper function to create mock qualifying results for testing
fn create_mock_qualifying_results() -> Vec<QualifyingResult> {
    vec![
        QualifyingResult {
            position: 1,
            driver: Driver {
                id: "driver1".to_string(),
                code: "DRV".to_string(),
                name: "Test Driver".to_string(),
                team: "Test Team".to_string(),
                number: 1,
            },
            q1: Some("1:20.123".to_string()),
            q2: Some("1:19.456".to_string()),
            q3: Some("1:18.789".to_string()),
        },
    ]
}

// Helper function to create mock practice results for testing
fn create_mock_practice_results() -> Vec<PracticeResult> {
    vec![
        PracticeResult {
            position: 1,
            driver: Driver {
                id: "driver1".to_string(),
                code: "DRV".to_string(),
                name: "Test Driver".to_string(),
                team: "Test Team".to_string(),
                number: 1,
            },
            time: Some("1:21.123".to_string()),
            laps: 25,
        },
    ]
}

// Helper to capture stdout for testing
#[allow(dead_code)]
struct StdoutCapture {
    pub captured_output: Arc<Mutex<Vec<u8>>>,
}

impl Write for StdoutCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.captured_output.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_simulate_handles_race_session_correctly() {
    // Setup
    let season = 2023;
    let gp = "monza";
    let session = "race";
    
    // Mock the data module
    let mut data_mock = MockDataModule::new();
    data_mock
        .expect_load_race_data()
        .with(eq(season), eq(gp))
        .times(1)
        .returning(move |s, g| Ok(create_mock_race(s, g)));

    // Call the simulate function with our mock
    let result = historical::simulate_with_data_module(season, gp, session, &data_mock);
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_simulate_handles_qualifying_session_correctly() {
    // Setup
    let season = 2023;
    let gp = "monza";
    let session = "qualifying";

    // Create mock qualifying results
    let mock_qualifying_results = create_mock_qualifying_results();
    
    // Mock the data module
    let mut data_mock = MockDataModule::new();
    data_mock
        .expect_load_qualifying_data()
        .with(eq(season), eq(gp))
        .times(1)
        .returning(move |_, _| Ok(mock_qualifying_results.clone()));

    // Call the simulate function with our mock
    let result = historical::simulate_with_data_module(season, gp, session, &data_mock);
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_simulate_handles_practice_session_correctly() {
    // Setup
    let season = 2023;
    let gp = "monza";
    let session = "fp1";
    let practice_number = 1;

    // Create mock practice results
    let mock_practice_results = create_mock_practice_results();
    
    // Mock the data module
    let mut data_mock = MockDataModule::new();
    data_mock
        .expect_load_practice_data()
        .with(eq(season), eq(gp), eq(practice_number))
        .times(1)
        .returning(move |_, _, _| Ok(mock_practice_results.clone()));

    // Call the simulate function with our mock
    let result = historical::simulate_with_data_module(season, gp, session, &data_mock);
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_simulate_handles_invalid_session_type() {
    // Setup
    let season = 2023;
    let gp = "monza";
    let invalid_session = "invalid_session";
    
    // Even with dependency injection, we can test the session validation directly
    let data_mock = MockDataModule::new();
    let result = historical::simulate_with_data_module(season, gp, invalid_session, &data_mock);
    
    // Verify that the error is appropriate
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Unknown session type"));
        assert!(e.to_string().contains(invalid_session));
    }
}

#[test]
fn test_simulate_race_handles_data_error() {
    // Setup
    let season = 2023;
    let gp = "nonexistent_gp";
    let session = "race";
    
    // Mock the data module to return an error
    let mut data_mock = MockDataModule::new();
    data_mock
        .expect_load_race_data()
        .with(eq(season), eq(gp))
        .times(1)
        .returning(|_, _| Err(anyhow::anyhow!("Race data not found")));
    
    // Call the simulate function with our mock
    let result = historical::simulate_with_data_module(season, gp, session, &data_mock);
    
    // Verify that the error is propagated
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Race data not found"));
    }
}

#[test]
fn test_simulate_qualifying_handles_data_error() {
    // Setup
    let season = 2023;
    let gp = "nonexistent_gp";
    let session = "qualifying";
    
    // Mock the data module to return an error
    let mut data_mock = MockDataModule::new();
    data_mock
        .expect_load_qualifying_data()
        .with(eq(season), eq(gp))
        .times(1)
        .returning(|_, _| Err(anyhow::anyhow!("Qualifying data not found")));
    
    // Call the simulate function with our mock
    let result = historical::simulate_with_data_module(season, gp, session, &data_mock);
    
    // Verify that the error is propagated
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Qualifying data not found"));
    }
}

#[test]
fn test_simulate_practice_handles_data_error() {
    // Setup
    let season = 2023;
    let gp = "nonexistent_gp";
    let session = "fp1";
    
    // Mock the data module to return an error
    let mut data_mock = MockDataModule::new();
    data_mock
        .expect_load_practice_data()
        .with(eq(season), eq(gp), eq(1))
        .times(1)
        .returning(|_, _, _| Err(anyhow::anyhow!("Practice data not found")));
    
    // Call the simulate function with our mock
    let result = historical::simulate_with_data_module(season, gp, session, &data_mock);
    
    // Verify that the error is propagated
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Practice data not found"));
    }
}