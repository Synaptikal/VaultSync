# 📦 VaultSync Project Specification

**Version:** 1.1

**Status:** Development Ready

**Architecture:** Native Appliance / Offline-First

---

## 1. Executive Summary

VaultSync is a specialized POS system for the Collectibles industry, supporting **TCG**, **Sports Cards**, **Comics**, **Merchandise**, and **Memorabilia**. It solves the "Triple-Threat" problem: **Extreme SKU counts** (100k+), **High Price Volatility**, and the need for **Seamless Trade-ins (Buylisting)** across diverse product categories. The system operates on a local-first mesh network, ensuring the shop never goes dark during internet outages.

---

## 2. Technical Architecture

### The "Local Mesh" Model

VaultSync does not rely on a central cloud server for real-time operations. It uses a **Native Appliance** model.

* **Database:** Embedded **SQLite** with **CRDT** (Conflict-free Replicated Data Types).
* **Networking:** **mDNS (ZeroConf)** for automatic device discovery (Peer-to-Peer).
* **Core Engine:** **Rust** for high-speed indexing and memory safety.
* **Frontend:** **Flutter** for cross-platform native performance (Windows/macOS/iOS/Android).

---

## 3. Database Schema (Simplified)

### `Global_Catalog` (Local Cache)

| Column | Type | Description |
| --- | --- | --- |
| `product_uuid` | UUID | Primary Key |
| `name` | String | Product Name |
| `category` | Enum | TCG, SportsCard, Comic, Apparel, etc. |
| `set_code` | String | e.g., "MTG-INV", "Fleer 86" |
| `barcode` | String | UPC/EAN |
| `metadata` | JSON | Category-specific fields (Team, Hero, Size) |

### `Local_Inventory`

| Column | Type | Description |
| --- | --- | --- |
| `inventory_uuid` | UUID | Primary Key |
| `product_uuid` | UUID | FK to Catalog |
| `condition` | Enum | NM, LP, GemMint, New, Used, etc. |
| `variant_type` | Enum | Foil, Signed, Graded, etc. |
| `quantity` | Int | On-hand count |

---

## 4. Feature Modules

### A. The Universal Buylist Engine

* **Unified Receipt:** Process buys and sells in a single transaction.
* **Multi-Category Support:** Handle raw cards, graded slabs, comics, and bulk merch in one flow.
* **Multi-Offer Logic:** Toggle between Cash (e.g., 50%) and Store Credit (e.g., 65%) with one click.

### B. Volatility Guard

* **Price Lock:** Automatically flags items for manager review if the market price has shifted >15% since the last local sync.
* **Offline Heartbeat:** Visible status bar showing the age of the price cache.

### C. Customer Kiosk Mode

* **Self-Intake:** Allows customers to scan their own items via a counter-top tablet, generating a digital manifest for the clerk to verify.

---

## 5. UI/UX Component Library

* **High-Contrast Design:** Optimized for low-light shop environments.
* **Tactile Targets:** Minimum 44px touch zones for fast, glove-friendly operation.
* **Dual-Stream View:** Split-screen interface separating "Shop Selling" from "Shop Buying."

---

## 6. Conflict Resolution & Discrepancy

In the event of an offline "Double Sale" (selling the same rare item on two different terminals before they sync):

* **Reconciliation Engine:** Flags the conflict upon reconnection.
* **Manager Dashboard:** Provides a "Decision Card" UI to choose which transaction to honor and which to refund/void.

---

## 7. Developer Roadmap

1. **Phase 1 (Core):** Build the Rust-based SQLite sync engine and mDNS discovery. (Completed)
2. **Phase 2 (Data):** Integrate TCGplayer/Cardmarket APIs and Protobuf compression. (In Progress)
3. **Phase 3 (Vision):** Implement AR-assisted scanning and Kiosk mode.
4. **Phase 4 (Bridge):** Multi-channel sync for eBay and TCGplayer storefronts.

---

## 8. Hardware Recommendations

* **Master Node:** Mac Mini or Intel NUC (Small Form Factor).
* **Terminals:** iPad Pro (11" or larger) for mobility.
* **Peripherals:** Zebra Thermal Printer, USB Postal Scale, Star Micronics Cash Drawer, Barcode Scanner.
