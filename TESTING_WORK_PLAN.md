# Testing Work Plan for macOS Audio Device Monitor

## Overview
This document outlines a systematic approach to adding comprehensive testing to the audio device monitor project. The plan is structured in three phases to gradually improve testability while maintaining functionality.

## Current State Analysis
- **Testable Components**: DevicePriorityManager, Config system, Device matching logic
- **Tightly Coupled**: DeviceController (CoreAudio system calls), Device enumeration
- **Mixed**: NotificationManager (has both pure logic and system calls)

## Phase 1: Unit Tests for Pure Logic Components
**Goal**: Add comprehensive unit tests for components that don't depend on system state

### Components to Test:
1. **DevicePriorityManager** 
   - Device selection algorithm
   - Weight-based prioritization
   - State tracking (current devices)
   - Edge cases (no devices, equal weights, disabled rules)

2. **Config System**
   - TOML parsing and validation
   - Default value generation
   - Backward compatibility migration
   - Error handling for invalid configs

3. **Device Matching Logic**
   - DeviceRule.matches() with all match types (exact, contains, starts_with, ends_with)
   - Case sensitivity
   - Special characters in device names
   - Edge cases (empty strings, whitespace)

4. **NotificationManager Pure Logic**
   - Configuration-based filtering
   - Notification content generation
   - Switch reason categorization

### Deliverables:
- `tests/unit/` directory structure
- Comprehensive test coverage for pure logic
- CI-ready test suite (`cargo test`)
- Documentation of test patterns

## Phase 2: Architecture Refactoring for Testability
**Goal**: Refactor tightly-coupled components to support dependency injection and mocking

### Refactoring Tasks:
1. **Extract System Interface Traits**
   - `AudioSystemInterface` trait for CoreAudio operations
   - `NotificationInterface` trait for system notifications
   - `ConfigLoader` trait for file system operations

2. **Dependency Injection**
   - Modify DeviceController to accept trait objects
   - Update constructors to support test doubles
   - Maintain backward compatibility for production use

3. **Mock Implementations**
   - `MockAudioSystem` for testing device operations
   - `MockNotificationSystem` for testing notifications
   - Test fixtures for common scenarios

4. **Separation of Concerns**
   - Extract pure business logic from system calls
   - Create facade pattern for complex operations
   - Improve error handling and propagation

### Deliverables:
- Trait-based architecture for system dependencies
- Mock implementations for all external systems
- Refactored components with dependency injection
- Maintained API compatibility

## Phase 3: Integration and End-to-End Testing
**Goal**: Test complete workflows and system interactions

### Integration Test Categories:
1. **Service Lifecycle Tests**
   - Service startup and shutdown
   - Configuration loading and validation
   - Signal handling (SIGHUP reload)

2. **Device Management Workflows**
   - Device discovery and enumeration
   - Priority-based selection
   - Automatic switching scenarios

3. **Configuration Management**
   - Hot reload functionality
   - Configuration validation and error handling
   - Backward compatibility scenarios

4. **Notification System**
   - End-to-end notification delivery
   - Configuration-based filtering
   - Error handling and fallback behavior

### Testing Infrastructure:
- Docker/container-based testing environment
- Test fixtures for various system states
- Property-based testing for configuration validation
- Performance benchmarks for critical paths

### Deliverables:
- `tests/integration/` directory with full workflow tests
- CI pipeline with automated testing
- Performance benchmarks and monitoring
- Documentation for adding new tests

## Success Criteria
- [ ] >90% code coverage for pure logic components
- [ ] All components testable in isolation
- [ ] Fast test suite (<30 seconds for unit tests)
- [ ] Reliable integration tests
- [ ] Clear testing documentation and patterns
- [ ] CI/CD pipeline with automated testing

## Implementation Notes
- Use `cargo fmt`, `cargo test`, `cargo clippy` after each step
- Create meaningful commits after each substantial change
- Don't commit planning files to git
- Update progress regularly during implementation
- Use subagents for detailed planning of each phase

## Risk Mitigation
- Maintain backward compatibility during refactoring
- Extensive testing of refactored components
- Gradual rollout of architectural changes
- Fallback plans for system-dependent testing