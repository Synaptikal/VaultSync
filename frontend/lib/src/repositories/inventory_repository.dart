import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:flutter/foundation.dart';
import '../api/generated/export.dart';
import '../datasources/local/inventory_local_datasource.dart';
import '../datasources/remote/inventory_remote_datasource.dart';
import '../services/api_exceptions.dart';
import 'base_repository.dart';

/// Inventory Repository (TASK-AUD-001e)
///
/// Single source of truth for inventory data.
/// Implements offline-first architecture.
///
/// Flow:
/// 1. Write operations: Save to local DB first (guaranteed)
/// 2. Sync to remote if online
/// 3. If offline or sync fails, queue for background sync
/// 4. Read operations: Prefer local cache, fallback to remote
///
/// This ensures:
/// - App works offline
/// - No data loss
/// - Eventual consistency
/// - Smooth UX (no waiting for network)

class InventoryRepository implements SyncableRepository<InventoryItem> {
  final InventoryRemoteDataSource _remote;
  final InventoryLocalDataSource _local;
  final Connectivity _connectivity;

  InventoryRepository({
    required InventoryRemoteDataSource remote,
    required InventoryLocalDataSource local,
    Connectivity? connectivity,
  })  : _remote = remote,
        _local = local,
        _connectivity = connectivity ?? Connectivity();

  /// Check if device is online
  Future<bool> get _isOnline async {
    final result = await _connectivity.checkConnectivity();
    return result != ConnectivityResult.none;
  }

  @override
  Future<List<InventoryItem>> getAll({int? limit, int? offset}) async {
    try {
      // Try remote first if online
      if (await _isOnline) {
        final items = await _remote.getAll(limit: limit, offset: offset);

        // Update local cache SAFELY (Prevent data loss)
        final dirtyUuids = await _local.getDirtyUuids();
        final safeItems =
            items.where((i) => !dirtyUuids.contains(i.inventoryUuid)).toList();

        await _local.insertBatch(safeItems);

        // Return combined list from local source (preserves local state)
        return await _local.getAll(limit: limit, offset: offset);
      }
    } on NetworkException {
      // Expected when offline, fall through to local
    } catch (e) {
      debugPrint('[InventoryRepository] Remote fetch failed: $e');
    }

    // Fallback to local cache
    return await _local.getAll(limit: limit, offset: offset);
  }

  @override
  Future<InventoryItem?> getById(String id) async {
    // Check local cache first (faster)
    final cached = await _local.getById(id);
    if (cached != null) {
      return cached;
    }

    // Not in cache, try remote
    try {
      if (await _isOnline) {
        final item = await _remote.getById(id);
        if (item != null) {
          await _local.insert(item);
          await _local.markSynced(item.inventoryUuid);
        }
        return item;
      }
    } catch (e) {
      debugPrint('[InventoryRepository] Remote getById failed: $e');
    }

    return null;
  }

  @override
  Future<InventoryItem> create(InventoryItem item) async {
    // 1. Save locally FIRST (guaranteed success)
    await _local.insert(item);

    // 2. Try to sync to server
    if (await _isOnline) {
      try {
        final synced = await _remote.create(item);

        // Update local with server-confirmed data
        await _local.update(synced);
        await _local.markSynced(synced.inventoryUuid);

        return synced;
      } on NetworkException {
        debugPrint('[InventoryRepository] Offline - item queued for sync');
      } catch (e) {
        debugPrint('[InventoryRepository] Sync failed but saved locally: $e');
      }
    }

    // Item saved locally, will sync in background
    return item;
  }

  @override
  Future<InventoryItem> update(InventoryItem item) async {
    // 1. Update locally FIRST
    await _local.update(item);

    // 2. Try to sync to server
    if (await _isOnline) {
      try {
        final synced = await _remote.update(item);

        // Update local with server-confirmed data
        await _local.update(synced);
        await _local.markSynced(synced.inventoryUuid);

        return synced;
      } on NetworkException {
        debugPrint('[InventoryRepository] Offline - update queued for sync');
      } catch (e) {
        debugPrint('[InventoryRepository] Sync failed but saved locally: $e');
      }
    }

    return item;
  }

  @override
  Future<void> delete(String id) async {
    // 1. Soft delete locally FIRST
    await _local.delete(id);

    // 2. Try to sync deletion to server
    if (await _isOnline) {
      try {
        await _remote.delete(id);

        // Deletion confirmed, can hard delete locally
        await _local.hardDelete(id);
      } on NetworkException {
        debugPrint('[InventoryRepository] Offline - deletion queued for sync');
      } catch (e) {
        debugPrint('[InventoryRepository] Sync failed but deleted locally: $e');
      }
    }
  }

  @override
  Future<List<InventoryItem>> search(String query) async {
    // Inventory search is typically by product, handled by getByProductUuid
    // For now, return all and filter locally
    final all = await _local.getAll();
    return all
        .where((i) =>
            i.productUuid.toLowerCase().contains(query.toLowerCase()) ||
            i.locationTag.toLowerCase().contains(query.toLowerCase()))
        .toList();
  }

  @override
  Future<List<InventoryItem>> refresh() async {
    // Force fetch from server
    if (!await _isOnline) {
      throw const NetworkException('Cannot refresh while offline');
    }

    final items = await _remote.getAll();

    // Update local cache SAFELY
    final dirtyUuids = await _local.getDirtyUuids();
    final safeItems =
        items.where((i) => !dirtyUuids.contains(i.inventoryUuid)).toList();

    await _local.insertBatch(safeItems);

    return await _local.getAll();
  }

  @override
  Future<int> syncPendingChanges() async {
    if (!await _isOnline) {
      return 0;
    }

    final unsynced = await _local.getUnsynced();
    int synced = 0;

    for (final item in unsynced) {
      try {
        if (item.deletedAt != null) {
          // Sync deletion
          await _remote.delete(item.inventoryUuid);
          await _local.hardDelete(item.inventoryUuid);
        } else {
          // Try to update (might be new or modified)
          try {
            await _remote.update(item);
          } catch (e) {
            // If update fails, try create
            await _remote.create(item);
          }
          await _local.markSynced(item.inventoryUuid);
        }
        synced++;
      } catch (e) {
        debugPrint(
            '[InventoryRepository] Failed to sync ${item.inventoryUuid}: $e');
      }
    }

    return synced;
  }

  @override
  Future<void> clearCache() async {
    await _local.clearAll();
  }

  // === SyncableRepository Methods ===

  @override
  Future<List<InventoryItem>> getUnsynced() async {
    return await _local.getUnsynced();
  }

  @override
  Future<List<InventoryItem>> getFailedSync() async {
    // TODO: Implement failed sync tracking
    return [];
  }

  @override
  Future<void> retrySync(String id) async {
    final item = await _local.getById(id);
    if (item == null) return;

    try {
      await _remote.update(item);
      await _local.markSynced(id);
    } catch (e) {
      debugPrint('[InventoryRepository] Retry sync failed for $id: $e');
      rethrow;
    }
  }

  // === Inventory-Specific Methods ===

  /// Get inventory items by product UUID
  Future<List<InventoryItem>> getByProductUuid(String productUuid) async {
    // Check local first
    final localResults = await _local.getByProductUuid(productUuid);

    if (await _isOnline) {
      try {
        final remoteResults = await _remote.getByProductUuid(productUuid);
        await _local.insertBatch(remoteResults);
        return remoteResults;
      } catch (e) {
        debugPrint('[InventoryRepository] Remote getByProduct failed: $e');
      }
    }

    return localResults;
  }

  /// Get low stock items
  Future<List<InventoryItem>> getLowStock({int threshold = 3}) async {
    // Check local first (fast)
    final localResults = await _local.getLowStock(threshold: threshold);

    if (await _isOnline) {
      try {
        final remoteResults = await _remote.getLowStock(threshold: threshold);
        return remoteResults;
      } catch (e) {
        debugPrint('[InventoryRepository] Remote getLowStock failed: $e');
      }
    }

    return localResults;
  }

  /// Get sync statistics
  Future<Map<String, int>> getSyncStats() async {
    final unsynced = await _local.getUnsynced();
    final total = await _local.count();

    return {
      'total': total,
      'unsynced': unsynced.length,
      'synced': total - unsynced.length,
    };
  }
}
