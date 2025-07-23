# Phase 1 Unit Testing Implementation Plan

## Overview
Implement comprehensive unit tests for pure logic components that don't depend on system state or CoreAudio APIs.

## Components to Test
1. **DevicePriorityManager** - Device selection based on weighted priority rules
2. **Config System** - TOML parsing, validation, backward compatibility  
3. **Device Matching Logic** - String matching with different MatchType variants
4. **NotificationManager Pure Logic** - Configuration-based filtering and content generation

## Implementation Steps

### Step 1: Test Infrastructure Setup
- Create test utilities and mock data builders
- Set up test directory structure (`tests/test_utils/`)
- Add AudioDevice and DeviceRule builders for test fixtures

### Step 2: Device Matching Tests (`tests/device_matching_tests.rs`)
- Test all MatchType variants: Exact, Contains, StartsWith, EndsWith
- Edge cases: empty strings, unicode, special characters, disabled rules
- Property-based testing for comprehensive coverage

### Step 3: Priority Manager Tests (`tests/priority_manager_tests.rs`)
- Device selection algorithm with multiple devices/rules
- Priority weight handling and state management
- Device type separation (input vs output)

### Step 4: Configuration Tests (`tests/config_tests.rs`)
- NotificationConfig backward compatibility migration
- Default value generation and validation
- TOML serialization/deserialization

### Step 5: Notification Logic Tests (`tests/notification_manager_tests.rs`)
- Configuration-based filtering logic
- Message content generation for different scenarios
- SwitchReason enum handling

### Step 6: Integration Tests (`tests/integration_pure_logic_tests.rs`)
- Component interaction without system calls
- End-to-end pure logic flows

## Success Criteria
- 100% pass rate for all tests
- >90% line coverage for tested modules
- Performance: <1ms priority selection for 100 devices
- Comprehensive edge case coverage

## Deliverables
6 commits with incremental testing implementation, establishing foundation for future system integration testing phases.