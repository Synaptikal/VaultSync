# VaultSync System Specification

**Version:** 2.1 (Enhanced with Business Context)
**Last Updated:** 2026-01-03
**Status:** Pre-Production / Quality Assurance

---

## 0. Business & Strategic Context
*Addressed based on `docs/vaultsync_spec_analysis.md` feedback*

### 0.1 Problem Statement
VaultSync addresses the "Triple-Threat" problem unique to the Collectibles (TCG/Sports Card) industry:
1.  **Extreme SKU Counts:** Shops manage 100,000+ unique low-value items (singles), far exceeding the capacity of generic POS systems like Square or Toast.
2.  **High Price Volatility:** Card values fluctuate daily. Shops need automated protection against buying high and selling low due to outdated data.
3.  **Complex Trade-Ins (Buylisting):** 40-60% of inventory comes from customers. The system must handle "Buylisting" (intake) as natively as selling, with support for cash vs. credit offers and condition grading.

### 0.2 Target Customer
*   **Primary:** Independent Card Shops (LGS - Local Game Stores) needing offline reliability and specialized inventory tools.
*   **Secondary:** Multi-location regional chains requiring decentralized inventory sync without expensive cloud ERPs.
*   **Differentiation:** Unlike cloud-reliant competitors (Shopify POS, Lightspeed), VaultSync utilizes a **Local-First Mesh** architecture. This ensures zero downtime during internet outages—critical for shops where "Friday Night Magic" events cannot stop due to connection loss.

### 0.3 Operational Goals
*   **Reliability:** 100% uptime for core Point of Sale and Buylist functions (Offline-capable).
*   **Data Integrity:** Eventual consistency with zero data loss during partition healing.
*   **Speed:** <200ms product lookup time against a 100k+ item database.

---

## 1. Technical Architecture & Current Functionality

### 1.1 Executive Summary
VaultSync is a specialized, offline-first Point of Sale (POS) and Inventory Management System. It leverages a distributed mesh architecture to ensure continuous operation, synchronizing data peer-to-peer across local terminals via mDNS discovery and TCP/HTTP replication.

### 1.2 System Topology

**Topology:** Decentralized Local Mesh (Peer-to-Peer)
**Discovery:** mDNS (ZeroConf/Bonjour) for automatic terminal discovery.
**Data Consistency:** Eventual Consistency using Vector Clocks and Last-Write-Wins (LWW) conflict resolution.

```mermaid
graph TD
    subgraph "Local Store Network"
        NodeA[Master Terminal (NodeA)] <-->|Sync Protocol (TCP/UDP)| NodeB[Pos Terminal 1 (NodeB)]
        NodeA <-->|Sync Protocol| NodeC[Pos Terminal 2 (NodeC)]
        NodeB <-->|Sync Protocol| NodeC
    end
    
    subgraph "External Services"
        PricingAPI[PriceCharting / TCGPlayer API]
        EmailGW[SMTP / Email Service]
    end

    NodeA -.->|HTTPS| PricingAPI
    NodeA -.->|SMTP| EmailGW
```

### 1.3 Technology Stack

#### Backend (Core Engine)
- **Language:** Rust (Edition 2021)
- **Web Framework:** Axum 0.7 (Tokio-based)
- **Database:** SQLite 3 (Embedded via sqlx 0.7) - *Chosen for single-file portability and high read performance.*
- **Sync Protocol:** Custom HTTP/JSON replication with Vector Clocks.
- **Security:** Argon2 (Hashing), JWT (Auth), TLS 1.3 (Transport).

#### Frontend (App)
- **Framework:** Flutter (Dart)
- **Platforms:** Windows (Primary), macOS, iOS, Android.
- **Communication:** REST API to local backend "sidecar".

---

## 2. Feature Inventory

### 2.1 Core Modules
| Feature | Functionality | Status |
| :--- | :--- | :--- |
| **Universal Buylist** | Unified intake interface. Supports Cash vs Store Credit toggles. Multi-category (Singles, Bulk, Slabs). | ✅ Implemented |
| **Point of Sale** | Standard checkout. Split payments, Tax calculation, Customer association. | ✅ Implemented |
| **Inventory Management** | CRUD for items. Supports conditions (NM/LP/MP), variants (Foil/Signed), and location tags. | ✅ Implemented |
| **Customer CRM** | Purchase history, Contact info, Tax-exempt status tracking. | ✅ Implemented |

### 2.2 Advanced Capabilities
| Feature | Functionality | Status |
| :--- | :--- | :--- |
| **Sync Engine** | P2P Mesh sync. Handles partition healing and conflict resolution automatically. | ✅ Implemented |
| **Volatility Guard** | Alerts staff if buy-price exceeds market threshold (requires external API). | ✅ Implemented |
| **Audit Logging** | Immutable ledger of all `INSERT/UPDATE/DELETE` ops for security/theft prevention. | ✅ Implemented |
| **Dynamic Pricing** | Integration with PriceCharting/Scryfall for live market data usage. | ✅ Implemented |
| **Event Management** | Tournament scheduling and player registration. | ✅ Implemented |

### 2.3 Hardware Support
- **Barcode/QR:** Generation (Code128, EAN13) and scanning integration.
- **Peripherals:** ESC/POS Receipt Printers, Cash Drawers.

---

## 3. Data Flow & Schemas

### 3.1 Primary Schemas (SQLite)

**Products (Catalog)**
- `product_uuid`, `name`, `category`, `set_code`
- `metadata` (JSON): Stores category-specifics (e.g., "Mana Cost" for MTG, "Team" for Sports).

**Inventory (Stock)**
- `inventory_uuid`, `product_uuid` (FK)
- `quantity`, `condition`, `variant_type`, `unit_price`, `location`

**Transactions (Ledger)**
- `transaction_uuid`, `customer_uuid` (FK)
- `type` (Sale/Buylist), `status` (Pending/Complete), `financial_snapshot`

### 3.2 Data Pipelines
1. **Sync Pipeline:**
   `Write -> DB Trigger -> Log Entry -> Broadcaster -> Peer Receiver -> Conflict Resolver -> Local DB`
2. **Pricing Pipeline:**
   `External API -> Price Cache (TTL 24h) -> Volatility Logic -> UI Alert`

---

## 4. Operational Maintenance & Reliability

### 4.1 Monitoring
- **Endpoints:** `/health` (Basic), `/health/detailed` (DB/Disk/Sync status).
- **Metrics:** Sync queue depth, replication latency (tracked internally).

### 4.2 Backup & Recovery
- **Strategy:** Local SQLite backups with rotation.
- **Schedule:** Automated (Configurable `BACKUP_INTERVAL_HOURS`).
- **Safety:** SHA-256 checksums on all backup files.
- **Recovery:** CLI/API restore capability. *Tested: Jan 2026.*

### 4.3 Security Posture
- **Access Control:** RBAC (Admin/Manager/Cashier).
- **Protection:** Parameterized queries (No SQLi), Rate limiting (Login), CORS policies.

---

## 5. Roadmap & Strategic Development
*Based on Technical Gap Analysis*

### Phase 1: Stability & Scale (Immediate - Q1 2026)
- [ ] **Load Testing:** Validate Sync Engine with 50+ concurrent nodes (Simulate large store events).
- [ ] **Migration Safety:** Implement and test robust schema migration rollback strategies.
- [ ] **Observability:** Dashboard for "Sync Health" visibility for non-technical managers.

### Phase 2: Market Expansion (Q2 2026)
- [ ] **E-Commerce Bridge:** Shopify/WooCommerce inventory sync (Two-way).
- [ ] **Marketplace Integration:** eBay/TCGPlayer listing management.

### Phase 3: Vision (Late 2026)
- [ ] **AR Scanning:** Vision-based fast intake for cards.
- [ ] **Cloud Federation:** Optional aggregation layer for multi-site chains.

---

## 6. Open Questions (To Be Answered via 12-Step Framework)
*The following areas require strategic input to finalize the specification:*

1.  **Business Model:** SaaS vs Perpetual? (Impacts licensing code requirements).
2.  **Scale Targets:** Max distinct SKUs supported? (Currently tested to ~10k, goal 100k?).
3.  **Team Velocity:** Current engineering capacity to deliver Phase 2?
4.  **Customer Feedback:** Validated pain points from Beta users?

---

## 7. API Reference (Snapshot)
`Base: /api/v1`

| Endpoint | Method | Function |
| :--- | :--- | :--- |
| `/products` | GET | Catalog text search |
| `/inventory` | POST | Adjust stock levels |
| `/transactions` | POST | Submit Sale/Buy |
| `/sync/push` | POST | P2P Replication endpoint |
| `/admin/audit` | GET | Security logs |
