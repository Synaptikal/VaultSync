1. The Core Database Schema
I. The Global Catalog (Read-Only Cache)
This table is updated via the cloud but lived locally. It contains every card ever printed.

card_uuid: Primary Key (UUID).

game_system: [Pokemon, MTG, YuGiOh, etc.].

set_code: (e.g., "BS-01" or "151").

collector_number: (e.g., "144/151").

metadata: JSON blob containing card text, artist, and legalities.

II. Local Inventory (The Store's "Physical" Reality)
This table tracks what is actually in the building.

inventory_uuid: Primary Key.

card_uuid: Foreign Key to Global Catalog.

variant_type: [Normal, Foil, Reverse Holo, First Edition, Stamped].

condition: [NM, LP, MP, HP, DMG].

quantity_on_hand: Integer.

location_tag: [Display Case, Backroom, Bin A, Consignment].

III. The Pricing Matrix
This is the most volatile table. It updates whenever the internet is live.

price_uuid: Primary Key.

card_uuid: Foreign Key.

market_mid: Decimal.

market_low: Decimal.

last_sync_timestamp: Used to trigger "Volatility Warnings."

2. Sync Logic: How "Offline" Works
The biggest technical hurdle is ensuring that when the internet or the local Wi-Fi cuts out, the data remains consistent.

The "Vector Clock" Sync
Each terminal keeps a "Version History." When Terminal B reconnects to the Main Hub, they compare histories.

Terminal A: "I sold 1x Charizard at 12:01 PM."

Terminal B: "I sold 1x Charizard at 12:05 PM."

Conflict Resolver: The Hub sees both, subtracts 2 from the total, and pushes the new "Current State" to all devices.

3. User Flow: The "Single-Screen" Buy/Sell
The UI must be designed so the clerk never has to leave the transaction screen to look up a price or check a customer's credit.

Step-by-Step UI Interaction:
Scan/Search Bar: Always active at the top.

The Split-Screen Bucket: * Left Side (Selling): Cards the customer is buying from the shop.

Right Side (Buying): Cards the shop is taking in from the customer.

The "Live Total": A footer that updates in real-time:

Customer Owes: $120.00

Trade-In Value: $85.00

Balance Due: $35.00

4. The "Store Credit" Ledger
In TCG shops, Store Credit is a local currency. VaultSync treats it like a bank account.

Transaction Locking: Store credit cannot be edited manually without a manager’s biometric or PIN.

History Tracking: Every cent of credit is tied to a specific transaction_uuid, so you can see exactly which cards were traded in to get that credit.

5. Implementation Plan: The "VaultSync" Dev Stack
To make this real, we would use the following technologies:

Language: Rust (for the high-speed local engine) or Go.

Frontend: Flutter (runs natively on Windows, macOS, and iPad with a single codebase).

Local Messaging: NATS or ZeroMQ (lightweight protocols for terminals to talk to each other without a server).