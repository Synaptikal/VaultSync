This document serves as the comprehensive Product Requirements Document (PRD) and Architectural Blueprint for **VaultSync**, the next-generation, offline-first POS designed specifically for the TCG and collectibles industry.

---

# 📄 Product Blueprint: VaultSync POS

**Tagline:** The Stock Exchange for Your Local Card Shop.

## 1. Executive Summary

VaultSync is a **Native Appliance** POS system. Unlike cloud-only competitors that fail during internet outages, or enterprise systems that are too complex for small retailers, VaultSync provides a local-first, zero-config environment that treats trading cards as volatile assets.

---

## 2. System Architecture (The "Local Mesh")

To avoid the complexity of Docker or manual server setups, VaultSync uses a **Native Peer-to-Peer** architecture.

* **Engine:** Native C++/Rust core for high-speed indexing.
* **Database:** Embedded **SQLite** with **CRDT** (Conflict-free Replicated Data Types) for seamless multi-device syncing without a central server.
* **Discovery:** **mDNS (ZeroConf)**—Devices find each other automatically on the local Wi-Fi.
* **Cloud Bridge:** Asynchronous HTTPS syncing. Sales data is backed up to the cloud only when a connection is available.

---

## 3. Core Feature Modules

### A. The "Intelligent" Inventory

Traditional POS systems use "Flat SKUs." VaultSync uses **Dynamic Variant Trees**.

* **Automatic Set Population:** Pre-loaded data for MTG, Pokémon, Yu-Gi-Oh!, Lorcana, and more.
* **The Condition Matrix:** One-click toggling between NM, LP, MP, HP, and Damaged, with automatic price scaling.
* **Bulk Management:** "Virtual Bins" for low-value cards, preventing search-result clutter while maintaining accurate stock counts.

### B. Two-Way Trade Engine (The Buylist)

The heart of a card shop is the trade-in desk.

* **Unified Transaction:** Process a sale and a trade-in on the same receipt.
* **Multi-Offer Logic:** Instant calculation of "Store Credit" vs. "Cash" offers based on store-wide margin rules.
* **USB Scale Integration:** "Weight-to-Value" for processing bulk commons (e.g., $5.00/lb).

### C. Market-Synced Pricing

* **Offline Price Cache:** Stores a rolling 24-hour snapshot of Market Mid, Market Low, and Foil Multipliers.
* **Volatility Alerts:** Visual "Heat" indicators on cards that have moved >10% in the last 12 hours.
* **Manual Overrides:** Quick-access "Sticker Price" vs. "Market Price" toggles.

---

## 4. User Experience (The "Clerk-First" Flow)

### The "3-Second Rule"

Every common action must be achievable in 3 seconds or less:

1. **Search:** "Pika 151" instantly pulls up *Pikachu from the 151 set*.
2. **Price Check:** Hover or tap to see the 30-day price trend.
3. **Add to Cart:** Single tap adds the item; a long-press opens condition/quantity options.

### Customer Kiosk Mode

* Turn any tablet into a **Self-Service Intake Station**.
* Customers scan their cards via the tablet's camera.
* Generates a "Trade Manifest" for the clerk to review, reducing data entry time by 90%.

---

## 5. Hardware Specifications

| Component | Recommendation |
| --- | --- |
| **Main Terminal** | Any Windows 10+ or macOS device (The "Hub"). |
| **Mobile Nodes** | iPads or Android Tablets for floor sales and kiosks. |
| **Scanner** | High-speed document scanner or mounted smartphone for "Pile Scanning." |
| **Labeling** | Dymo/Zebra support for 1x1 inch QR codes for high-value "Slabs." |

---

## 6. Failure Mode Resilience (The "Pre-Mortem" Fixes)

* **Internet Fails:** System continues exactly as before. Prices stay locked to the last known cache.
* **Power Outage:** Local database performs "Journaling" to ensure the very last transaction is never corrupted.
* **Simultaneous Sale:** If two clerks sell the last copy of a card at the exact same time while offline, the system flags a **"Physical Reconciliation Error"** for the manager the moment the devices sync.

---

## 7. Development Roadmap

### Phase 1: The "Iron" Core (Months 1-3)

* Build the native SQLite/CRDT sync engine.
* Develop the "Zero-Config" network discovery.

### Phase 2: The "Library" (Months 4-6)

* Import global card databases.
* Build the Buylist logic and "Store Credit" ledger.

### Phase 3: "Vision" (Months 7-9)

* Implement camera-based card recognition.
* Launch the "Customer Kiosk" interface.

### Phase 4: "The Bridge" (Months 10-12)

* Multi-channel sync (eBay, TCGplayer integration).
* End-to-end beta testing with 10 local game stores.
