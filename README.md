# VaultSync

[![Status](https://img.shields.io/badge/Status-Pre--Production-yellow)](https://github.com/Start-Impulse/VaultSync)
[![Backend](https://img.shields.io/badge/Backend-Rust-orange)](https://www.rust-lang.org/)
[![Frontend](https://img.shields.io/badge/Frontend-Flutter-blue)](https://flutter.dev/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

**Next-Generation Offline-First POS & Inventory System for Collectibles Stores.**

VaultSync is a specialized Point of Sale (POS) and inventory management platform designed specifically for the unique needs of Local Game Stores (LGS) and collectibles shops. It solves the "Triple-Threat" problem of the industry: extreme SKU counts, high price volatility, and complex trade-in (buylist) workflows.

Unlike cloud-reliant competitors, VaultSync utilizes a **Local-First Mesh Architecture**, ensuring 100% uptime and zero latency even during internet outages—critical for high-volume store events.

---

## 🚀 Key Features

*   **🔌 Offline-First Core:** Full functionality without an internet connection. Data syncs peer-to-peer when devices reconnect.
*   **📦 High-Volume Inventory:** Optimized for tracking 100,000+ unique singles with variants (Foil, Graded, Condition).
*   **🛒 Universal Buylist:** Native "Trade-In" mode to handle customer sells efficiently (Cash vs. Store Credit offers).
*   **💸 Dynamic Pricing:** Integration with pricing providers (e.g., PriceCharting, Scryfall) for real-time market value alerts.
*   **🔄 P2P Mesh Sync:** Decentralized synchronization using Vector Clocks and conflict resolution to keep multiple terminals in sync.
*   **📅 Event Management:** Built-in tournament scheduling and player registration.
*   **🔒 Security & Auditing:** Role-based access control (RBAC) and immutable audit logs for all transactions.

---

## 🏗️ Technical Architecture

*   **Backend:** Rust (Axum, SQLx, SQLite)
    *   *Why Rust?* Type safety, performance, and memory safety for the core sync engine.
    *   *Why SQLite?* Portable, single-file database ideal for local-first deployment.
*   **Frontend:** Flutter (Mobile/Desktop)
    *   *Target Platforms:* Windows (Primary POS), macOS, tablet/mobile companions.
*   **Protocol:** Custom HTTP/TCP replication protocol using Vector Clocks for eventual consistency.

---

## 🛠️ Getting Started

### Prerequisites

*   [Rust Toolchain](https://rustup.rs/) (1.75+)
*   [Flutter SDK](https://docs.flutter.dev/get-started/install) (3.0+)
*   MinGW (for Windows SQLite support) or `build-essential` (Linux)

### 1. Backend Setup

The backend serves as the local "node" for the application.

```bash
# Navigate to project root
cd VaultSync

# Install dependencies and build
cargo build

# Run the server (Defaults to port 3000)
cargo run
```

### 2. Frontend Setup

The frontend connects to the local backend node.

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
flutter pub get

# Run the application
# Note: Ensure the backend is running first!
flutter run
```

---

## 🧪 Testing

We maintain a rigorous testing standard to ensure data integrity.

**Backend Tests:**
```bash
cargo test
```

**Frontend Tests:**
```bash
cd frontend
flutter test
```

---

## 📚 Documentation

Detailed technical documentation is available in the project structure:

*   [Technical Specification](TECHNICAL_SPECIFICATION.md) - Deep dive into architecture and data flow.
*   [API Documentation](docs/) - Endpoint references.
*   [Frontend Progress](FRONTEND_PROGRESS.md) - Tracking UI implementation status.

---

## 🗺️ Roadmap

*   **Phase 1 (Current):** Stability, Sync Engine Verification, and production hardening.
*   **Phase 2:** E-commerce integrations (Shopify/eBay) and Multi-site federation.
*   **Phase 3:** Advanced automation (AR Scanning).

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
