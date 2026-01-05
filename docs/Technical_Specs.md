# VaultSync Technical Specifications

## 1. System Architecture

### Backend (Core Engine)
- **Language**: Rust
- **Framework**: Axum (Web Framework), Tokio (Async Runtime)
- **Database**: SQLite (Embedded, Single-file)
- **Authentication**: JWT (Stateless), Argon2 (Password Hashing)
- **Service Layer**:
    - `InventoryService`: Manages local stock.
    - `TransactionService`: Handles sales, buys, and trades.
    - `PricingService`: Syncs with external market data (mocked).
    - `BuylistService`: Calculates buy prices based on condition.

### Frontend (POS Terminal)
- **Framework**: Flutter (Dart)
- **Platforms**: Windows, macOS, Android, iOS
- **Navigation**: GoRouter
- **State Management**: setState (MVP), Provider (Planned)
- **Networking**: `http` package, `flutter_secure_storage` for tokens.
- **Initialization**: Platform-specific files (e.g., `windows/`, `macos/`) are generated on demand via `flutter create .` if missing.

## 2. Cross-Platform POS Application

### Features
- **Product Registration**: Add products to the global catalog and local inventory.
- **POS Terminal**: Select products, manage cart, process transactions.
- **Customer Management**: CRM features, store credit tracking.
- **Inventory Management**: Real-time stock tracking across categories (TCG, Sports, Comics, etc.).

### UI/UX
- Responsive design adapting to Desktop (Landscape) and Mobile (Portrait).
- Material Design 3 components.

## 3. Security

- **Data Encryption**: User passwords hashed with Argon2.
- **API Security**: All protected endpoints require `Authorization: Bearer <token>` header.
- **Transport**: HTTPS recommended for production (currently HTTP for local dev).

## 4. Offline Functionality & Sync (Planned)

- **Local-First Architecture**: The frontend will use a local database (e.g., `sqflite`) to store data.
- **Sync Queue**: Operations performed offline are added to a queue.
- **Replication**: When online, the queue is replayed against the backend.
- **Conflict Resolution**: Last-write-wins or manual intervention policies.

## 5. Deployment

### CI/CD
- **GitHub Actions**: Automated testing (`cargo test`) and linting (`clippy`) on every push.
- **Build Artifacts**: Future workflows will build `vaultsync.exe` (Windows) and `VaultSync.app` (macOS).

### Auto-Update
- **Update Server**: A central server hosting the latest binaries.
- **Client Logic**: The application checks for updates on startup and downloads the installer if available.
