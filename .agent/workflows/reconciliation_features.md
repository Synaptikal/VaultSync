---
description: Implementation of Inventory Reconciliation, Conflict Resolution, and Blind Count Audit features.
---

# Inventory Reconciliation & Audit Workflow

This workflow tracks the implementation of the "Reconciliation Engine" and "Blind Count" features defined in `InfoOnUI.md`.

## Phase 1: Database Schema
1. **Migrations**
   - [x] Add `Inventory_Conflicts` table.
     - `conflict_uuid` (PK)
     - `product_uuid` (FK)
     - `conflict_type` (Enum: Oversold, Price_Mismatch, Credit_Anomaly, Physical_Miscount)
     - `terminal_ids` (JSON Array)
     - `expected_quantity` (Int)
     - `actual_quantity` (Int)
     - `resolution_status` (Boolean / Enum: Pending, Resolved, Ignored)
     - `resolved_by` (User UUID)
     - `resolution_notes` (Text)
     - `created_at` (Timestamp)
     - `resolved_at` (Timestamp)

## Phase 2: Backend Logic
2. **Audit Service**
   - [x] Create `src/audit/mod.rs`.
   - [x] Implement `Blind Count` logic:
     - `submit_blind_count(location, items)` -> Generates `Inventory_Conflicts`.
   - [ ] Implement `Reconciliation` logic:
     - `detect_conflicts()` -> Scheduled job to check SyncLog vs Inventory state.

3. **API Endpoints**
   - [x] `GET /api/audit/conflicts` (List active conflicts)
   - [x] `POST /api/audit/resolve` (Resolve a conflict)
   - [x] `POST /api/audit/submit-blind-count` (Merged start/submit)

## Phase 3: Integration
4. **Wiring**
   - [x] Register `AuditService` in `main.rs` and `AppState`.
   - [x] Add routes to `api/handlers.rs`.
