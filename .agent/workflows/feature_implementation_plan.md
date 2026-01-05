---
description: Comprehensive feature implementation plan for VaultSync enhancements including Multi-Provider Pricing, Serialized Inventory, Dynamic Pricing, Wants List, and Event Management.
---

# VaultSync Feature Implementation Plan

This workflow outlines the step-by-step implementation of five key systems to enhance the VaultSync POS.

## Phase 1: Core Pricing & Provider Architecture
**Goal**: Expand the system to support multiple TCGs and Collectible types beyond simple hardcoded logic, and introduce dynamic pricing rules.

1. **Multi-Provider Pricing Engine**
   - [x] Refactor `PricingProvider` trait in `src/pricing/providers.rs` to support Category-based dispatch.
   - [x] Implement `ProviderFactory` or `PricingRegistry` in `src/pricing/mod.rs`.
   - [x] Create stub implementations for `PokemonProvider`, `SportsCardProvider`, and `GenericProvider`.
   - [x] Update `BuylistService` to select the correct provider based on `Product.category`.

2. **Refined Dynamic Pricing Rules**
   - [x] Define `PricingRule` struct in `src/core/mod.rs` (matchers: Category, Set, Rarity; actions: Multiplier, Flat Adjustment).
   - [x] Create `RuleEngine` in `src/pricing/rules.rs` to evaluate a `Product` + `Condition` against a list of rules.
   - [x] Integrate `RuleEngine` into `BuylistService::calculate_item_price` to replace hardcoded 0.5/0.65 multipliers.

## Phase 2: Inventory & Data Model Enhancements
**Goal**: Support high-value items where individual tracking matters (Serialized Inventory).

3. **Serialized Inventory System**
   - [x] Update `InventoryItem` in `src/core/mod.rs` to support a `serialized_data` field (Optional Struct) or linked entity.
     - Needs: `certification_number`, `grader` (PSA/BGS), `images` (Vec<String> paths), `specific_price`.
   - [x] Update `Database` schema to support storing this extra data.
   - [x] Update `InventoryService` to handle "Add Unique Item" vs "Add Bulk Item".

## Phase 3: Customer & Engagement Features
**Goal**: Drive sales and specific acquisitions through customer-focused features.

4. **Distributed Wants List**
   - [x] Create `WantsList` and `WantsItem` entities in `src/core/mod.rs`.
   - [x] Add `Database` methods to CRUD Wants Lists.
   - [x] Implement `WantsMatchingService` in `src/buylist/matcher.rs`.
     - Logic: When `process_buylist_transaction` occurs, check incoming items against active Wants Lists.
     - Output: Generate a `Notification` or `Alert` (just a log/event for now).

5. **Event & Tournament Management**
   - [x] Create `Event`, `EventParticipant` entities in `src/core/mod.rs`.
   - [x] Implement `EventService` to handle:
     - Registration (creates a `Transaction`).
     - Prizing (deducts from `Inventory`).
   - [x] Add API/Service methods to list and manage events.

## Phase 4: Integration & Cleanup
6. **Final Verification**
   - [x] Verify all new services are registered in `main.rs`.
   - [x] Ensure `sync` module handles new entities (Wants, Events, Serialized data).
   - [x] Run comprehensive tests.
