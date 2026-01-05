# Frontend Refactoring Plan - Production-Grade Implementation

**Date:** 2026-01-04  
**Objective:** Align Flutter frontend with production-ready backend  
**Estimated Duration:** 2 weeks  

---

## Executive Summary

The backend is now production-ready (v0.2.0) with:
- ✅ Robust conflict resolution system
- ✅ Inventory audit capabilities
- ✅ 100+ documented API endpoints
- ✅ Comprehensive error handling

The frontend needs refactoring to:
1. Replace naive HTTP client with robust middleware
2. Implement proper offline-first architecture
3. Build conflict resolution UI
4. Add audit/blind count screens
5. Improve error handling and state management

---

## Current State Assessment

### ✅ What's Working
- Basic UI structure exists
- Navigation with go_router
- Provider-based state management foundation
- Local storage with SQLite

### ❌ Critical Issues (From Audit)
1. **API Layer:** Manual HTTP client with no interceptors
2. **State Management:** Mixed concerns, optimistic failures
3. **Offline Logic:** Silent error swallowing
4. **Missing Features:** Conflict resolution, blind count UIs
5. **Error Handling:** Generic messages, no retry logic

---

## Refactoring Phases

### 📘 PHASE 1: API & Networking (Days 1-3)

#### 1.1 Dependency Management
**File:** `pubspec.yaml`

Add:
```yaml
dependencies:
  dio: ^5.4.0              # HTTP client with interceptors
  pretty_dio_logger: ^1.3.1 # Request/response logging
  connectivity_plus: ^5.0.0 # Network state detection
  retry: ^3.1.2            # Exponential backoff
```

#### 1.2 Create ApiClient (Replace ApiService)
**File:** `lib/src/services/api_client.dart`

Features:
- ✅ Dio-based with interceptors
- ✅ Automatic token refresh on 401
- ✅ Request/response logging
- ✅ Standardized error types
- ✅ Retry logic with exponential backoff

**Reference:** Use `refactored_api_client.dart` as base

#### 1.3 Error Handling
**File:** `lib/src/services/api_exceptions.dart`

Create typed exceptions:
```dart
sealed class ApiException implements Exception {
  final String message;
  const ApiException(this.message);
}

class NetworkException extends ApiException { }
class AuthenticationException extends ApiException { }
class ValidationException extends ApiException { }
class ConflictException extends ApiException { }
```

#### 1.4 Update All Providers
Replace `ApiService` usage with new `ApiClient` across:
- `product_provider.dart`
- `inventory_provider.dart`
- `transaction_provider.dart`
- `customer_provider.dart`
- `sync_provider.dart`

---

### 📗 PHASE 2: Repository Pattern (Days 4-5)

#### 2.1 Create Repository Interfaces
**File:** `lib/src/repositories/base_repository.dart`

```dart
abstract class Repository<T> {
  Future<List<T>> getAll();
  Future<T?> getById(String id);
  Future<T> create(T item);
  Future<T> update(T item);
  Future<void> delete(String id);
}
```

#### 2.2 Implement Repositories
Create for each domain:
- `ProductRepository` - Remote + Local datasources
- `InventoryRepository`
- `TransactionRepository`
- `SyncRepository`

**Pattern:**
```dart
class ProductRepository implements Repository<Product> {
  final ProductRemoteDataSource remote;
  final ProductLocalDataSource local;
  final ConnectivityService connectivity;
  
  Future<Product> create(Product product) async {
    // Save locally first
    await local.insert(product);
    
    if (await connectivity.isOnline()) {
      try {
        final synced = await remote.create(product);
        await local.markSynced(synced.id);
        return synced;
      } catch (e) {
        // Already saved locally, queue for sync
        await syncQueue.add(product);
        return product;
      }
    }
    return product;
  }
}
```

#### 2.3 Datasource Abstraction
**Remote:** `lib/src/datasources/remote/`
- Use `ApiClient` for network calls
- Handle pagination
- Parse responses

**Local:** `lib/src/datasources/local/`
- SQLite operations
- Caching logic
- Offline queue

---

### 📙 PHASE 3: Offline Queue & Sync (Days 6-7)

#### 3.1 Create SyncQueue Service
**File:** `lib/src/services/sync_queue_service.dart`

Features:
- Persistent queue (SQLite table)
- Operation types: CREATE, UPDATE, DELETE
- Retry with exponential backoff
- Status tracking: PENDING, SYNCING, FAILED

#### 3.2 Background Sync Worker
**File:** `lib/src/services/background_sync_service.dart`

Use `workmanager` package for background tasks:
```dart
void callbackDispatcher() {
  Workmanager().executeTask((task, inputData) async {
    await SyncQueueService().processQueue();
    return Future.value(true);
  });
}
```

#### 3.3 Sync Status UI
**File:** `lib/src/shared/widgets/sync_status_indicator.dart`

Visual indicator:
- ✅ Green: All synced
- 🟡 Yellow: Syncing (N pending)
- 🔴 Red: Failed (tap to retry)

#### 3.4 Update Providers
Refactor `ProductProvider` to use `ProductRepository`:
- Remove direct API calls
- Remove optimistic sync logic
- Let repository handle offline/online

---

### 📕 PHASE 4: Conflict Resolution UI (Days 8-9)

#### 4.1 Fetch Conflicts Endpoint Integration
**File:** `lib/src/services/api_client.dart`

Add method:
```dart
Future<List<SyncConflict>> getPendingConflicts() async {
  final response = await dio.get('/api/sync/conflicts');
  return (response.data as List)
    .map((json) => SyncConflict.fromJson(json))
    .toList();
}
```

#### 4.2 Conflict Model
**File:** `lib/src/models/sync_conflict.dart`

```dart
class SyncConflict {
  final String conflictUuid;
  final String resourceType;
  final String resourceUuid;
  final String conflictType;
  final Map<String, dynamic> localState;
  final Map<String, dynamic> remoteState;
  final String detectedAt;
  
  // fromJson, toJson, etc.
}
```

#### 4.3 Conflict Resolution Screen
**File:** `lib/src/features/sync/screens/conflict_resolution_screen.dart`

UI Components:
- **Conflict List:** Show pending conflicts
- **Detail View:** Side-by-side comparison
  - Left: Local state (This Terminal)
  - Right: Remote state (Terminal B)
- **Action Buttons:**
  - 🟢 Keep Local
  - 🔵 Keep Remote
  - 🟡 Manual Edit

#### 4.4 Resolution Logic
```dart
Future<void> resolveConflict(String uuid, String strategy) async {
  await apiClient.post('/api/sync/conflicts/resolve', data: {
    'conflict_uuid': uuid,
    'resolution': strategy, // 'LocalWins' | 'RemoteWins'
  });
  
  // Refresh conflicts list
  await loadConflicts();
}
```

---

### 📔 PHASE 5: Inventory Audit UI (Days 10-11)

#### 5.1 Blind Count Screen
**File:** `lib/src/features/inventory/screens/blind_count_screen.dart`

Features:
- **Scanner Mode:** Barcode entry without showing DB quantity
- **Item List:** Track scanned items
- **Submit:** Send to `/api/audit/submit-blind-count`

#### 5.2 Discrepancy Review
**File:** `lib/src/features/inventory/screens/audit_discrepancies_screen.dart`

Show results:
```dart
ListView(
  children: discrepancies.map((d) => Card(
    child: ListTile(
      title: Text(d.productName),
      subtitle: Text('Expected: ${d.expected}, Found: ${d.actual}'),
      trailing: Text(
        d.variance > 0 ? '+${d.variance}' : '${d.variance}',
        style: TextStyle(
          color: d.variance > 0 ? Colors.green : Colors.red,
        ),
      ),
    ),
  )).toList(),
)
```

#### 5.3 Inventory Conflict Integration
Connect to backend's `Inventory_Conflicts` table for persistent tracking.

---

### 📓 PHASE 6: Polish & Testing (Days 12-14)

#### 6.1 Error Messages
Replace generic "Something went wrong" with:
- Network errors: "No internet connection. Changes saved locally."
- Validation: "Invalid barcode format"
- Conflict: "Price was updated by another terminal. Please review."

#### 6.2 Loading States
Add proper loading indicators:
- Shimmer loading for lists
- Progress indicators for sync
- Pull-to-refresh support

#### 6.3 Integration Testing
**File:** `test/integration_test.dart`

Test scenarios:
- Offline product creation → Online sync
- Conflict detection → Resolution
- Blind count → Discrepancy review
- Token expiration → Auto-refresh

#### 6.4 Performance
- Implement pagination for large lists
- Add caching layer (flutter_cache_manager)
- Optimize image loading
- Lazy load screens

---

## File Structure (After Refactoring)

```
lib/src/
├── datasources/
│   ├── local/
│   │   ├── product_local_datasource.dart
│   │   ├── inventory_local_datasource.dart
│   │   └── sync_queue_local_datasource.dart
│   └── remote/
│       ├── product_remote_datasource.dart
│       └── inventory_remote_datasource.dart
├── models/
│   ├── product.dart
│   ├── inventory_item.dart
│   ├── sync_conflict.dart
│   └── audit_discrepancy.dart
├── repositories/
│   ├── base_repository.dart
│   ├── product_repository.dart
│   ├── inventory_repository.dart
│   └── sync_repository.dart
├── services/
│   ├── api_client.dart             ← NEW (replaces api_service.dart)
│   ├── api_exceptions.dart         ← NEW
│   ├── connectivity_service.dart   ← NEW
│   ├── sync_queue_service.dart     ← NEW
│   ├── background_sync_service.dart ← NEW
│   └── storage_service.dart
├── providers/
│   ├── product_provider.dart       ← REFACTORED
│   ├── inventory_provider.dart     ← REFACTORED
│   ├── sync_provider.dart          ← NEW
│   └── auth_provider.dart
├── features/
│   ├── sync/
│   │   ├── screens/
│   │   │   ├── conflict_resolution_screen.dart  ← NEW
│   │   │   └── sync_status_screen.dart
│   │   └── widgets/
│   │       └── conflict_card.dart               ← NEW
│   └── inventory/
│       ├── screens/
│       │   ├── blind_count_screen.dart          ← NEW
│       │   └── audit_discrepancies_screen.dart  ← NEW
│       └── widgets/
│           └── discrepancy_card.dart            ← NEW
└── shared/
    └── widgets/
        └── sync_status_indicator.dart            ← NEW
```

---

## Dependencies to Add

```yaml
# pubspec.yaml additions
dependencies:
  # Networking
  dio: ^5.4.0
  pretty_dio_logger: ^1.3.1
  connectivity_plus: ^5.0.0
  retry: ^3.1.2
  
  # Background tasks
  workmanager: ^0.5.1
  
  # Caching
  flutter_cache_manager: ^3.3.1
  
  # UI
  shimmer: ^3.0.0
  pull_to_refresh: ^2.0.0
  
  # Already present (verify versions):
  provider: ^6.0.0
  go_router: ^12.0.0
  sqflite: ^2.3.0
```

---

## Success Criteria

### Must Have (MVP)
- [x] Backend APIs functional
- [ ] Dio-based ApiClient with interceptors
- [ ] Repository pattern implemented
- [ ] Offline queue with retry logic
- [ ] Conflict resolution UI functional
- [ ] Blind count audit screens
- [ ] Proper error messages

### Nice to Have
- [ ] Background sync worker
- [ ] Advanced caching
- [ ] Shimmer loading states
- [ ] Performance optimization

### Quality Gates
- [ ] No direct API calls in UI code
- [ ] All network errors handled gracefully
- [ ] Offline functionality works without crashes
- [ ] Token refresh automatic
- [ ] Integration tests pass

---

## Timeline

| Phase | Days | Deliverable |
|-------|------|-------------|
| 1. API & Networking | 1-3 | ApiClient, Error types |
| 2. Repository Pattern | 4-5 | All repositories |
| 3. Offline Queue | 6-7 | Sync queue, background worker |
| 4. Conflict Resolution | 8-9 | UI screens, resolution logic |
| 5. Inventory Audit | 10-11 | Blind count, discrepancies |
| 6. Polish & Testing | 12-14 | Error messages, tests |

**Total: 2 weeks (14 days)**

---

## Risk Mitigation

### Risk: Breaking existing functionality
**Mitigation:** Incremental refactoring, feature flags

### Risk: SQLite schema changes
**Mitigation:** Migration scripts, version checks

### Risk: Performance degradation
**Mitigation:** Profiling, lazy loading, pagination

---

## Next Steps

1. **Review and approve this plan**
2. **Install dependencies** (Phase 1.1)
3. **Create ApiClient** (Phase 1.2)
4. **Refactor first provider** (ProductProvider)
5. **Test offline scenarios**

---

**Ready to proceed?** Let's start with Phase 1: API & Networking refactoring.
