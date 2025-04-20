// Export modules for use in tests and as a library
pub mod data;
pub mod models;
pub mod simulator;
pub mod utils;

// Re-export main simulator modules for convenience
pub use simulator::historical;
pub use simulator::prediction;