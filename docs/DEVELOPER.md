# VaultSync Developer Guide

## Overview

This guide provides information for developers looking to contribute to or extend the VaultSync POS system. The system is built in Rust and designed with modularity, testability, and extensibility in mind.

## Architecture Overview

### High-Level Architecture

VaultSync follows a modular architecture with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐
│   UI Layer      │    │   Service Layer │
│                 │    │                 │
│  CLI Interface  │◄───┤  Transaction    │
│                 │    │  Processing     │
└─────────────────┘    │                 │
                       ├─────────────────┤
                       │  Inventory      │
                       │  Management     │
                       ├─────────────────┤
                       │  Pricing        │
                       │  Service        │
                       ├─────────────────┤
                       │  Buylist        │
                       │  Engine         │
                       └─────────────────┘
                              │
                       ┌─────────────────┐
                       │   Core Layer    │
                       │                 │
                       │  Data Models    │
                       │  Serialization  │
                       └─────────────────┘
                              │
                       ┌─────────────────┐
                       │  Data Layer     │
                       │                 │
                       │  Database       │
                       │  (SQLite)       │
                       └─────────────────┘
```

### Module Structure

```
src/
├── core/           # Data structures and type definitions
├── database/       # SQLite database operations
├── network/        # mDNS discovery and network communication
├── pricing/        # Market price management
├── inventory/      # Inventory management system
├── sync/           # CRDT synchronization
├── ui/             # User interface (CLI)
├── buylist/        # Buylist and trade-in functionality
├── transactions/   # Transaction processing
├── errors/         # Error types and handling
└── main.rs         # Application entry point
```

## Setting Up Development Environment

### Prerequisites

1. Install Rust toolchain (1.70 or higher)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup install stable
   rustup default stable
   ```

2. Install Git
   ```bash
   # On Windows
   winget install Git.Git

   # On macOS
   brew install git

   # On Linux (Ubuntu/Debian)
   sudo apt install git
   ```

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/your-organization/vaultsync.git
cd vaultsync

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

### Development Tools

Recommended tools for development:

- **IDE**: VS Code with rust-analyzer extension, or IntelliJ IDEA with Rust plugin
- **Formatter**: rustfmt (included with Rust toolchain)
- **Linter**: clippy (included with Rust toolchain)
- **Dependency management**: cargo

## Core Concepts

### Data Models

All core data structures are defined in the `core` module. These structures are shared across all other modules.

#### Card
Represents a unique trading card in the system:
- `card_uuid`: Unique identifier for the card
- `game_system`: The game system (e.g., "Pokemon", "MTG")
- `set_code`: The set the card belongs to
- `collector_number`: The collector number (e.g., "001/102")
- `metadata`: JSON blob with additional card information

#### InventoryItem
Represents a physical item in inventory:
- `inventory_uuid`: Unique identifier for this inventory item
- `card_uuid`: References the card definition
- `variant_type`: Physical variant (Normal, Foil, etc.)
- `condition`: Physical condition (NM, LP, MP, HP, DMG)
- `quantity_on_hand`: Current quantity available
- `location_tag`: Where in the store the item is located

### CRDT Synchronization

The system implements Conflict-Free Replicated Data Types (CRDTs) for peer-to-peer synchronization:

1. **Vector Clocks**: Track causality between operations
2. **Operation-based CRDTs**: Track individual changes
3. **Convergence**: Eventually consistent across all peers

## Module Details

### Database Module

Handles all SQLite operations with a focus on:

- **Thread Safety**: Uses `Arc<Mutex<Connection>>` for concurrent access
- **CRUD Operations**: Create, Read, Update, Delete for all entities
- **Transaction Support**: Database transactions for consistency
- **Connection Pooling**: Efficient connection management

Key functions:
- `initialize_tables()`: Creates all required database tables
- `insert_card()`: Adds or updates a card in the catalog
- `insert_inventory_item()`: Adds or updates inventory
- `get_inventory_items()`: Retrieves all inventory items

### Network Module

Implements mDNS-based peer discovery:

- **mDNS Discovery**: Automatic peer detection using `mdns-sd` crate
- **Service Registration**: Self-advertising on the network
- **Event Handling**: Asynchronous event processing
- **Service Resolution**: Resolving discovered services to IP addresses

### Pricing Module

Manages market price data:

- **Price Sync**: Synchronization with external pricing APIs
- **Volatility Detection**: Identifies price changes that require review
- **Price Safety**: Flags for manager approval when prices change significantly
- **Caching**: Offline price cache for resilience

### Inventory Module

Handles inventory operations:

- **Condition Tracking**: Manages card conditions (NM, LP, MP, HP, DMG)
- **Quantity Management**: Tracks available quantities
- **Location Tagging**: Tracks where items are located
- **Variant Management**: Handles different card variants

### Sync Module

Implements CRDT-based synchronization:

- **Change Tracking**: Records all changes with vector timestamps
- **Peer Synchronization**: Syncs changes between connected peers
- **Conflict Resolution**: Handles conflicts using business rules
- **Eventual Consistency**: Ensures all peers converge to the same state

## Adding New Features

### Adding a New Data Model

1. Define the structure in `src/core/mod.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewModel {
    pub uuid: Uuid,
    pub field1: String,
    pub field2: i32,
}
```

2. Add database operations in `src/database/mod.rs`:
```rust
impl Database {
    pub fn insert_new_model(&self, model: &NewModel) -> Result<()> {
        // Implementation
    }
    
    pub fn get_new_model(&self, uuid: Uuid) -> Result<Option<NewModel>> {
        // Implementation
    }
}
```

3. Create a service module for business logic (optional but recommended)

### Adding a New Service

1. Create a new module: `src/new_service/mod.rs`
2. Define the service structure:
```rust
pub struct NewService {
    db: Arc<Database>,
    // other dependencies
}

impl NewService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    pub fn do_something(&self) -> Result<()> {
        // Implementation
    }
}
```

3. Register the module in `src/lib.rs`:
```rust
pub mod new_service;
```

4. Add error types in `src/errors/mod.rs` if needed

### Adding New CLI Commands

1. Modify the CLI interface in `src/ui/mod.rs`
2. Add command handling logic
3. Update help text
4. Add integration tests

## Testing Strategy

### Test Structure

```
tests/
├── integration_test.rs    # Integration tests for module interactions
└── [module]_test.rs       # Module-specific tests (future)
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests with higher log level
RUST_LOG=debug cargo test
```

### Test Guidelines

1. **Unit Tests**: Test individual functions in the same file
2. **Integration Tests**: Test interactions between modules
3. **Use In-Memory Database**: For test isolation
4. **Test Error Cases**: Verify proper error handling
5. **Test Concurrency**: Where applicable

Example test:
```rust
#[tokio::test]
async fn test_inventory_add_item() {
    let db = database::initialize_test_db().unwrap();
    let service = InventoryService::new(db);
    
    let item = InventoryItem {
        // ... item initialization
    };
    
    service.add_item(item).unwrap();
    
    let items = service.get_all_items().unwrap();
    assert_eq!(items.len(), 1);
}
```

## Error Handling

### Error Types

The system uses a custom error type defined in `src/errors/mod.rs`:

```rust
#[derive(Error, Debug)]
pub enum VaultSyncError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlite::Error),
    // ... other error variants
}
```

### Error Handling Guidelines

1. **Be Specific**: Use specific error types for different scenarios
2. **Chain Errors**: Use `#[from]` attribute for error chaining
3. **User-Friendly Messages**: Provide clear error messages
4. **Log Errors**: Use tracing for error logging

## Logging and Monitoring

### Logging Strategy

The system uses the `tracing` crate for structured logging:

- **ERROR**: Critical issues that stop functionality
- **WARN**: Issues that might affect operation
- **INFO**: Important operational events
- **DEBUG**: Detailed information for troubleshooting

### Logging Guidelines

1. **Be Consistent**: Use consistent log message formats
2. **Include Context**: Include relevant identifiers in logs
3. **Avoid Sensitive Data**: Don't log sensitive information
4. **Performance**: Avoid expensive operations in logging

Example:
```rust
tracing::info!(
    "Added item to inventory: card={}, condition={:?}, quantity={}",
    item.card_uuid,
    item.condition,
    item.quantity_on_hand
);
```

## Performance Considerations

### Database Performance

- Use prepared statements for repeated queries
- Index frequently queried columns
- Use transactions for multiple related operations
- Consider connection pooling for high concurrency

### Memory Management

- Use `Arc` for shared data between threads
- Implement proper cloning strategies
- Consider memory usage in long-running operations

### Concurrency

- Use async/await for I/O operations
- Use appropriate synchronization primitives
- Avoid blocking operations in async contexts

## Contributing

### Code Standards

1. **Formatting**: Use `rustfmt` for consistent formatting
2. **Linting**: Use `clippy` for code quality
3. **Documentation**: Document public APIs
4. **Testing**: Include tests for new functionality

### Git Workflow

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make changes with clear, concise commit messages
3. Run tests: `cargo test`
4. Format code: `cargo fmt`
5. Lint code: `cargo clippy`
6. Push and create a pull request

### Pull Request Guidelines

- Include a clear description of changes
- Add or update tests as needed
- Update documentation if applicable
- Reference related issues

## Common Development Tasks

### Adding a New API Endpoint

1. Define the data structure
2. Add database operations
3. Create service method
4. Add CLI/command interface
5. Write tests
6. Update documentation

### Modifying Database Schema

1. Add migration logic (future enhancement)
2. Update database module
3. Update data structures
4. Test with existing data
5. Update tests

### Extending Network Functionality

1. Update network module
2. Consider security implications
3. Add error handling
4. Test network scenarios
5. Update documentation

## Debugging

### Common Debugging Commands

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run tests with debug output
cargo test -- --nocapture

# Format code
cargo fmt

# Check for warnings
cargo clippy

# Check for unused dependencies
cargo machete  # Requires cargo install cargo-machete
```

### Debugging Tips

1. **Use tracing**: Add debug logs liberally during development
2. **Test in isolation**: Use in-memory databases for isolated testing
3. **Check logs**: Examine logs for error patterns
4. **Use debugger**: VS Code supports Rust debugging

## Building and Deployment

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build for specific target
cargo build --target x86_64-unknown-linux-musl
```

### Cross-Platform Builds

The system is designed to be cross-platform:
- Windows: x86_64-pc-windows-msvc
- macOS: x86_64-apple-darwin
- Linux: x86_64-unknown-linux-gnu

### Creating a Release

1. Update version in `Cargo.toml`
2. Update changelog
3. Run all tests
4. Build release binaries
5. Create GitHub release

## Future Enhancements

### Planned Features

- **Web UI**: Web-based interface for advanced management
- **Mobile App**: Companion app for inventory management
- **Advanced Analytics**: Reporting and analytics dashboard
- **Integration APIs**: REST API for external integrations
- **User Management**: Multi-user access control
- **Enhanced Sync**: More sophisticated conflict resolution

### Architecture Improvements

- **Microservices**: Potential separation of services
- **Message Queue**: For improved async processing
- **Caching Layer**: Redis or similar for performance
- **Monitoring**: Metrics and health checks

## Troubleshooting

### Common Issues

1. **Database Locking**: Use in-memory DB for tests, ensure single writer
2. **Network Discovery**: Check firewall and multicast settings
3. **Memory Usage**: Monitor for memory leaks in long-running operations
4. **Sync Conflicts**: Implement proper conflict resolution strategies

### Getting Help

- Check the existing issues on GitHub
- Run `cargo clippy` and `cargo fmt` for common issues
- Review the existing tests for implementation patterns
- Ask questions in pull requests or issues