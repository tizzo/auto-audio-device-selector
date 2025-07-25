pub mod adapters;
pub mod integration;
pub mod traits;

// Mock implementations for testing
#[cfg(test)]
pub mod mocks;

// Re-export traits and adapters for easy access
pub use adapters::*;
pub use traits::*;

// Re-export mocks when testing
#[cfg(test)]
pub use mocks::*;
