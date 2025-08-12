pub mod adapters;
pub mod integration;
pub mod traits;

// Mock implementations for testing (available for both unit and integration tests)
#[cfg(any(test, feature = "test-mocks"))]
pub mod mocks;

// Re-export traits and adapters for easy access
pub use adapters::*;
pub use traits::*;

// Re-export mocks for testing
#[cfg(any(test, feature = "test-mocks"))]
pub use mocks::*;
