# Contributing to VaultSync

Thank you for your interest in contributing to VaultSync! This document outlines the process for contributing to the project.

## Code of Conduct
Please be respectful and considerate in all interactions.

## Development Workflow

1.  **Fork the Repository:** Create your own fork of the project.
2.  **Create a Branch:** Create a feature branch for your changes (e.g., `feature/sync-optimization`).
3.  **Code:** Implement your changes. Ensure you follow the Rust and Dart style guides.
    *   **Backend:** run `cargo fmt` and `cargo clippy` before committing.
    *   **Frontend:** run `dart format .`
4.  **Test:**
    *   Run backend tests: `cargo test`
    *   Run frontend tests: `flutter test`
5.  **Commit:** Use clear, descriptive commit messages.
6.  **Pull Request:** Submit a PR to the `main` branch. Provide a detailed description of your changes.

## Coding Standards

### Rust (Backend)
- Use idiomatic Rust patterns.
- Handle all errors (avoid `unwrap()` in production code).
- Document public APIs using doc comments `///`.
- Use async/await for I/O bound operations.

### Flutter (Frontend)
- Follow Effective Dart guidelines.
- Use strong typing.
- Separate UI code from business logic (use BLoC or Provider pattern).

## Database Migrations
- Use `sqlx` migrations found in `migrations/`.
- Never modify an existing migration file; create a new one.

## Reporting Issues
- Use the detailed issue template.
- Include reproduction steps and environment details.
