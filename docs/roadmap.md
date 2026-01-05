# Kodaxa VaultSync Roadmap

## Overview

Kodaxa VaultSync is an offline-first POS system designed for the TCG, Sports Cards, Comics, and Collectibles industry. The system operates as a "shop edge computer" where each shop owns its own vault: your data, your pricing, your inventory. The system features LAN-local sync between terminals over mDNS with vector timestamps, with optional cloud relay/backup for multi-location operations.

## Product Architecture

### Core Components
- **Rust "vault core"**: Secure, performant backend engine
- **SQLite local "ledger"**: Embedded database for local data ownership
- **P2P "sync fabric"**: mDNS-based peer-to-peer synchronization infrastructure

### Product SKUs
- **VaultSync POS**: Cashier terminal UI for transaction processing with full offline capability
- **VaultSync Kiosk**: Self-service buy/trade intake with durable local cache for customer interactions
- **VaultSync Hub**: Back-office inventory and pricing console with advanced reporting

## Current State ("Now")

### Core Functionality
- **Offline POS**: Basic offline functionality with SQLite local storage
- **Rust+SQLite**: Solid foundation with embedded database
- **Single-shop operations**: Inventory management, basic transactions
- **Simple buy/sell**: Basic transaction processing with inventory updates

### Technology Stack
- **Backend**: Rust 1.70+ with Axum web framework
- **Database**: SQLite embedded database
- **Frontend**: Flutter cross-platform application
- **Networking**: mDNS for zero-configuration device discovery
- **Logging**: Tracing crate for structured logging

### LAN Sync Implementation
- **LAN sync between terminals over mDNS with vector timestamps**: Local network discovery and synchronization using mDNS for automatic device detection
- **Basic change tracking**: Vector timestamps track modifications across devices
- **SQLite-based local storage**: Each terminal maintains its own copy of data for offline operation

### TCG-Specific Features
- **Basic condition rules**: Simple condition-based pricing (NM, LP, MP, HP, DMG)
- **Cash vs credit multipliers**: Different rates for cash vs store credit transactions
- **Inventory consolidation**: Basic trade-in processing

### Flutter Frontend
- **REST API integration**: Flutter app communicates with Rust backend via HTTP endpoints
- **Basic offline capability**: Limited offline functionality when backend is unavailable

## Future Development ("Next")

### Synchronization
- **Optional cloud relay**: Cloud-based relay for multi-location sync and backup
- **Robust peer-to-peer reconciliation**: Complete multi-device synchronization with proper conflict handling
- **Deterministic merge rules**: Per-entity conflict resolution for inventory quantities, last-modified timestamps, and transaction records:
  - Inventory quantities use additive merging
  - Last-modified timestamps determine conflict resolution for metadata
  - Transaction records follow first-write-wins policy

### TCG-Specific Buylist Features
- **Set-specific buy rates**: Different pricing multipliers based on specific card sets and rarities
- **Advanced bulk rules**: Volume-based pricing for multiple card transactions
- **Condition matrix optimization**: Sophisticated condition-based pricing with market volatility adjustments
- **Manager approval workflows**: Flagged items requiring manual review for unusual price movements
- **Rules by condition**: Advanced condition-based pricing rules
- **Cash vs credit multipliers**: Sophisticated multiplier systems

### External Pricing Integration
- **Real API integrations**: Integration with TCGplayer, Cardmarket, and other pricing APIs
- **Volatility detection**: Advanced price movement monitoring with alerts
- **Market-synced pricing**: Real-time price updates with safety status calculations

### Flutter Frontend Enhancements
- **Durable local cache**: Implement proper offline-first architecture with local SQLite database (e.g., using sqflite)
- **Sync queue**: Operations performed offline are queued for synchronization when connectivity is restored
- **Conflict behavior**: Clear conflict resolution strategies when offline changes conflict with server updates
- **Local-first data model**: Mirror backend data model locally with proper synchronization protocols

### Advanced Features
- **Kiosk mode**: Self-service customer intake station
- **Enhanced trade-in workflows**: Complete buylist engine with advanced pricing algorithms
- **Advanced reporting**: Business intelligence and analytics features
- **Multi-location support**: Centralized management of multiple store locations

## Technical Implementation Priorities

### Phase 1: Core Stability
1. Complete basic transaction processing
2. Stabilize inventory management system
3. Implement proper database transaction handling

### Phase 2: Synchronization
1. Implement vector-timestamped sync with deterministic merge rules
2. Add robust peer-to-peer reconciliation
3. Complete LAN sync functionality

### Phase 3: Advanced Features
1. Integrate external pricing APIs
2. Implement advanced buylist functionality
3. Add durable local cache to Flutter frontend

### Phase 4: Scale & Enhancement
1. Implement cloud relay capability
2. Add multi-location support
3. Enhance reporting and analytics