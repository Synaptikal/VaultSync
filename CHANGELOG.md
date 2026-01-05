# Changelog

All notable changes to the VaultSync project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-01-04

### Added
- **Conflict Resolution System**: Full implementation of CRDT-based conflict detection and resolution
  - New `Sync_Conflicts` and `Conflict_Snapshots` tables (Migration 26)
  - `SyncService::record_sync_conflict()` to persist concurrent modifications
  - Enhanced `get_sync_conflicts()` to query dedicated conflict tables instead of string-searching logs
  - `resolve_sync_conflict()` with support for "LocalWins" and "RemoteWins" strategies
  - Side-by-side state comparison support for UI (local_state vs remote_state)
- **Inventory Audit System**: Production-ready blind count functionality
  - New `Inventory_Conflicts` table (Migration 27)
  - `InventoryService::submit_blind_count()` for physical inventory verification
  - `AuditDiscrepancy` core type for variance tracking
  - Integration with `AuditService` for conflict persistence
- **Frontend Middleware Prototype**: `RefactoredApiClient.dart`
  - Dio-based HTTP client with centralized interceptors
  - Automatic token refresh on 401 responses
  - Standardized error handling and logging
  - Type-safe methods for new conflict resolution endpoints
- **Integration Tests**: Comprehensive test suite for conflict resolution
  - Concurrent modification detection tests
  - Blind count audit workflow tests
  - Version vector comparison tests
  - Conflict persistence and resolution tests

### Changed
- **Database Layer**: Refactored conflict handling from "toy" log-search implementation to robust schema
  - `get_sync_conflicts` now returns rich DTOs with both local and remote state snapshots
  - Better separation of sync conflicts (concurrent edits) vs audit conflicts (physical discrepancies)
- **SyncService**: Enhanced to record all concurrent modifications as conflicts before auto-resolution
  - Conflicts are now "caught" and persisted for manual review even if auto-resolved
  - Added detailed logging for conflict detection and resolution

### Fixed
- **Critical**: Eliminated "silent loss" scenario where concurrent edits were auto-merged without audit trail
- **High**: Sync conflicts are now persistently stored instead of only appearing in transient logs
- Unused variable warning in `database/mod.rs` (conflict resolution logic)

### Security
- Conflict resolution requires proper authentication (conflicts exposed via protected API routes)
- Audit trail for all conflict resolutions (resolved_by, resolution_strategy, timestamps)

## [0.1.1] - 2026-01-08

### Added
- Schema migrations for Phase 14 (Business Core, Transaction Extensions).
- Quantity validation in returns processing to prevent fraud.

### Changed
- Refactored `src/database/mod.rs` to move schema definitions to `src/database/migrations.rs`.
- Streamlined `ReturnsService` to remove unused fields.
- Consolidated transaction logic by removing dead code in `TransactionService`.

### Fixed
- Undefined behavior in alerting service tests (unsafely mocking Database).
- Compilation errors related to missing struct fields in Phase 14.
- Redundant return processing logic between `TransactionService` and `ReturnsService`.

### Added
- Initial implementation of VaultSync POS system
- Core data models for TCG inventory management
- SQLite database with CRDT synchronization
- mDNS-based network discovery
- Inventory management system with condition tracking
- Pricing service with market data integration
- Buylist and trade-in functionality
- Transaction processing system
- Comprehensive error handling
- Structured logging with tracing
- Integration tests for core functionality

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - 2025-01-01

### Added
- Initial project structure and architecture
- Core data models (Card, InventoryItem, PriceInfo, Transaction)
- SQLite database implementation with all required tables
- Network discovery using mDNS protocol
- Pricing service with simulated market data integration
- Inventory management with condition tracking (NM, LP, MP, HP, DMG)
- Buylist engine for trade-ins and purchases
- Transaction processing system for sales, buys, trades, and returns
- CRDT-based synchronization between devices
- Custom error types with comprehensive error handling
- Structured logging using the tracing crate
- Integration tests covering core functionality
- CLI interface for basic operations
- Documentation for developers and users
- Deployment guide for production environments

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- Database locking issues in concurrent operations
- Type mismatches between modules
- Missing trait implementations for data structures

### Security
- N/A

## Development Milestones

### Phase 1: Core Infrastructure
- [x] Project structure and build system
- [x] Core data models
- [x] Database schema and operations
- [x] Basic error handling

### Phase 2: Core Functionality
- [x] Inventory management
- [x] Pricing system
- [x] Network discovery
- [x] Basic synchronization

### Phase 3: Advanced Features
- [x] Buylist and trade-in functionality
- [x] Transaction processing
- [x] Comprehensive error handling
- [x] Logging and monitoring

### Phase 4: Quality Assurance
- [x] Integration tests
- [x] Documentation
- [x] Deployment guides
- [x] Developer guides

## Future Releases

### Planned for 0.2.0
- Web-based user interface
- Advanced reporting and analytics
- Enhanced customer management
- Integration with external market APIs
- Barcode scanning support

### Planned for 0.3.0
- Mobile application for inventory management
- Advanced synchronization algorithms
- User authentication and permissions
- Cloud backup options
- Enhanced security features

### Planned for 1.0.0 (Production Ready)
- Complete UI/UX implementation
- All planned integrations
- Comprehensive security implementation
- Performance optimizations
- Production deployment automation
- Complete documentation suite