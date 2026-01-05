# VaultSync

VaultSync is a next-generation Point of Sale (POS) and Inventory Management system designed for collectibles stores. It features a robust offline-first architecture, enabling seamless operation even without an internet connection.

## Architecture

- **Backend**: Rust (Axum, SQLx, SQLite)
- **Frontend**: Flutter (Mobile/Desktop)
- **Database**: SQLite (Sync-enabled)

## Features

- **Offline-First**: Full functionality without internet; syncs when online.
- **Inventory Management**: Track products, variants (foil, graded, etc.), and conditions.
- **Point of Sale**: Process transactions with customer tracking.
- **Pricing Integration**: Modular pricing providers (e.g., Scryfall).
- **Synchronization**: Bi-directional sync using Vector Clocks and Merkle Trees (conceptually).

## Getting Started

### Backend

1. Install Rust.
2. Run the server:
   ```bash
   cargo run
   ```
   The server will start on `http://localhost:3000`.

### Frontend

1. Install Flutter.
2. Run the app:
   ```bash
   cd frontend
   flutter run
   ```

## API Documentation

### Sync

- `POST /api/sync/push`: Push local changes to the server.
- `GET /api/sync/pull?since=<timestamp>`: Pull remote changes from the server.

### Products

- `GET /api/products`: List products (supports pagination and search).
- `POST /api/products`: Create a new product.

### Inventory

- `GET /api/inventory`: List inventory items.
- `POST /api/inventory`: Add items to inventory.

## Testing

Run backend tests:
```bash
cargo test
```

Run frontend tests:
```bash
cd frontend
flutter test
```
