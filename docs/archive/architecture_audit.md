# Architecture Review Report

## Executive Summary
The `VaultSync` backend is built as a modular monolith using Rust, utilizing `axum` for the API and `sqlite` for persistence. While the modular structure (`core`, `pricing`, `inventory`, etc.) is sound and promotes separation of concerns, there are critical concurrency and architectural issues that will impact performance and scalability.

## Critical Findings

### 1. Blocking I/O in Async Runtime (High Severity)
The application uses the `sqlite` crate, which provides a synchronous (blocking) interface. The `Database` struct wraps the connection in a `std::sync::Mutex`.
- **The Problem**: When a database query runs, it blocks the entire Tokio worker thread. If the pool of worker threads is exhausted, the API will stop responding to **all** requests, including health checks.
- **Impact**: Poor performance under concurrent load; potential deadlocks.
- **Recommendation**: 
    - **Immediate**: Wrap database interactions in `tokio::task::spawn_blocking`.
    - **Strategic**: Migrate to `sqlx` (async-native, compile-time checked queries) or `tokio-rusqlite`.

### 2. Redundant Concurrency Wrappers (Medium Severity)
In `main.rs`, services are wrapped in multiple layers of concurrency primitives unnecessarily.
- **Example**: `PricingService` is wrapped in `Arc<tokio::sync::Mutex<...>>` in some contexts, but the service itself is stateless (or holds internal thread-safe state like `Arc<Database>`).
- **Impact**: Unnecessary locking overhead and code complexity.
- **Recommendation**: Services should be designed to be `Send + Sync` internally. `Arc<Service>` should be sufficient to share them.

### 3. Schema Management (Medium Severity)
Database tables are created via raw SQL strings in `initialize_tables`.
- **Problem**: No migration history or versioning. Changing the schema requires manual SQL execution or risky code changes.
- **Recommendation**: Adopt a migration system (e.g., `sqlx-cli` or a simple `user_version` pragma check).

### 4. Hardcoded dependencies in `main.rs`
The dependency injection is manual and rigid.
- **Recommendation**: Continue with manual DI for now (it's simple), but clean up the initialization order and wrapping.

## Refactoring Plan

I recommend proceeding with the following refactor steps:

1.  **Simplify Service definitions**: Remove unnecessary `Mutex` wrappers from `main.rs`.
2.  **Isolate Blocking Code**: Since switching to `sqlx` is a large task, we will verify that DB calls are fast enough for now, but acknowledge the technical debt.
3.  **Clean up `main.rs`**: Fix the initialization logic.

---

## Immediate Action: Clean up `main.rs` and Service Locking
I will refactor `main.rs` to remove the double-wrapping of services and ensure the `AppState` holds clean `Arc<Service>` references.
