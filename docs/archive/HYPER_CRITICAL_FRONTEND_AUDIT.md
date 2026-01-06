# Hypercritical Audit: Frontend & Middleware

**Date:** 2026-01-03
**Scope:** Frontend (Flutter), API Layer, State Management
**Status:** **CRITICALLY DEFICIENT**

## Executive Summary
While the backend has undergone rigorous hardening, the frontend implementation interacts with it using a brittle, non-production-ready architecture. The separation of concerns is blurry, error handling is generic, and the "Offline-First" capability—a core selling point of VaultSync—is implemented dangerously optimistically.

The frontend is currently "playing it safe" by implementing happy-path UI features without the robust plumbing required for a reliable POS system.

## 1. Middleware / API Layer Failures
**Severity: CRITICAL**

The `ApiService` (`lib/src/services/api_service.dart`) is a naive HTTP wrapper that fails fundamental architectural requirements for a distributed system.

*   **No Interceptors**: Authentication, logging, and error handling are manually repeated across methods. There is no centralized way to handle token expiration (401), leading to a guaranteed crash or stuck state when tokens expire.
*   **Boilerplate Heavy**: Every API call repeats `Uri.parse`, header construction, and status code checking. This creates a massive surface area for bugs (e.g., forgetting to check a status code).
*   **Generic Error Swallowing**: Exceptions are thrown as `Exception('Failed to...: ${statusCode}')`. The UI cannot distinguish between a "Validation Error" (400), "Auth Error" (401), or "Server Error" (500). This forces the UI to show generic "Something went wrong" messages, frustrating users.
*   **Performance Bottleneck**: The token is read from secure storage *on every single request*. This is an unnecessary I/O operation that will cause jank in high-frequency operations (like scanning items).

**Recommendation**:
*   Replace manual `http` calls with a robust client like `Dio` or a custom wrapper with Interceptors.
*   Implement a centralized `ApiClient` that handles Auth headers (with in-memory caching), Request Logging, and standardized Error Parsing.

## 2. State Management & Offline Logic
**Severity: HIGH**

The "Offline-First" implementation in `ProductProvider` (`lib/src/providers/product_provider.dart`) is dangerous.

*   **Logic Mixing**: The Provider acts as a Controller, Service, and Repository. It manually parses JSON, manages caching, and handles UI state.
*   **Optimistic Sync Failures**: The `addInventory` logic tries to sync, catches *any* error, and falls back to local storage without differentiation. If the *code* fails (e.g., JSON parsing error), it hides the bug and "saves" corrupted data locally.
*   **Silent Sync Failures**: Background sync swallows errors. If a user adds 50 items offline and sync fails for one, they are never notified, leading to inventory drift.
*   **Inefficient Reactivity**: Adding an item triggers a full `loadProducts()` reload. As inventory grows to 10k+ items, adding a single item will freeze the UI while it re-fetches or re-reads the entire database.

**Recommendation**:
*   Implement a proper Repository pattern that abstracts `LocalDatasource` and `RemoteDatasource`.
*   Use a synchronization queue with visible status (Pending, Syncing, Failed) so users verify their data is safe.
*   Optimize list updates: Append locally instead of reloading the world.

## 3. Feature Gaps vs. Spec
**Severity: MEDIUM**

The features described in `InfoOnUI.md` ("Conflict Resolution Cards", "Blind Count Audit") appear missing or rudimentary in the current file structure.

*   **Conflict Resolution**: No dedicated screen or logic found for the "Three-way check" (Expected vs. Reported vs. Physical).
*   **Blind Count**: No evidence of the "Blind Count" mode in the inventory module.

**Recommendation**:
*   These are high-value differentiators. They must be prioritized over "basic CRUD" screens.

## 4. Code Quality & Type Safety
**Severity: MEDIUM**

*   **Hardcoded Strings**: API endpoints are string literals. A typo in one file breaks a feature silently until runtime.
*   **Manual JSON**: Manual `jsonDecode` and casting is fragile. Codegen is present (`api/generated`) but the `ApiService` often manually decodes anyway before passing to `fromJson`.


## 5. Backend Gaps Blocking UI Features
**Status:** ✅ **RESOLVED (Backend Schema & Logic Implemented)**

The backend implementation for "Reconciliation" and "Conflict Resolution" is now complete and matches the `InfoOnUI.md` specification.

*   **Missing Schema**: The `Sync_Conflicts` table (specified with `conflict_uuid`, `affected_sku`, `terminal_ids`) does not exist.
*   **Hack Implementation**: `get_sync_conflicts` currently queries the raw `Sync_Log` for strings containing "Conflict". This is a "toy" implementation that cannot support the rich "Side-by-Side" comparison UI described in the requirements.
*   **No Blind Count logic**: There is no backend support for the "Blind Count" audit mode.

**Action Taken (Remediation Complete):**
*   **Schema**: Added `Sync_Conflicts`, `Conflict_Snapshots` and `Inventory_Conflicts` tables (Migrations 26 & 27).
*   **Logic**: 
    *   Implemented `SyncService.record_sync_conflict` to persist concurrent edits.
    *   Implemented `AuditService` support for blind counts.
    *   Updated `get_sync_conflicts` to query the dedicated table.
*   **Next Step**: Frontend needs to switch to `RefactoredApiClient.dart` (Prototype Created) to utilize these new endpoints.

## Remediation Plan

1.  **Refactor API Layer**: Introduce `Dio`, centralized error handling, and mapped Exceptions.
2.  **Hardening Sync**: Implement a robust `SyncQueue` that persists pending mutations and retries them with exponential backoff and user visibility.
3.  **Backend Support**: Implement `Sync_Conflicts` table and `submit_blind_count` endpoints to unblock the frontend features.
4.  **Implement Core Differentiators**: Build the "Conflict Resolution" UI to prove the backend's CRDT capabilities.
