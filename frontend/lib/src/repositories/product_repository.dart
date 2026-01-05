import 'package:connectivity_plus/connectivity_plus.dart';
import '../api/generated/export.dart';
import '../datasources/local/product_local_datasource.dart';
import '../datasources/remote/product_remote_datasource.dart';
import '../services/api_exceptions.dart';
import 'base_repository.dart';

/// Product Repository (PHASE 2 - Repository Pattern)
///
/// Single source of truth for product data.
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

class ProductRepository
    implements FilterableRepository<Product>, SyncableRepository<Product> {
  final ProductRemoteDataSource _remote;
  final ProductLocalDataSource _local;
  final Connectivity _connectivity;

  ProductRepository({
    required ProductRemoteDataSource remote,
    required ProductLocalDataSource local,
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
  Future<List<Product>> getAll({int? limit, int? offset}) async {
    try {
      // Try remote first if online
      if (await _isOnline) {
        final products = await _remote.getAll(limit: limit, offset: offset);

        // Update local cache SAFELY (Prevent data loss)
        final dirtyUuids = await _local.getDirtyUuids();
        final safeProducts =
            products.where((p) => !dirtyUuids.contains(p.productUuid)).toList();

        await _local.insertBatch(safeProducts);

        // Return combined list from local source (preserves local state)
        return await _local.getAll(limit: limit, offset: offset);
      }
    } on NetworkException {
      // Expected when offline, fall through to local
    } catch (e) {
      // Other errors, log but fall through to local
      print('[ProductRepository] Remote fetch failed: $e');
    }

    // Fallback to local cache
    return await _local.getAll(limit: limit, offset: offset);
  }

  @override
  Future<Product?> getById(String id) async {
    // Check local cache first (faster)
    final cached = await _local.getById(id);
    if (cached != null) {
      return cached;
    }

    // Not in cache, try remote
    try {
      if (await _isOnline) {
        final product = await _remote.getById(id);
        if (product != null) {
          await _local.insert(product);
          await _local.markSynced(product.productUuid);
        }
        return product;
      }
    } catch (e) {
      print('[ProductRepository] Remote getById failed: $e');
    }

    return null;
  }

  @override
  Future<Product> create(Product product) async {
    // 1. Save locally FIRST (guaranteed success)
    await _local.insert(product);

    // 2. Try to sync to server
    if (await _isOnline) {
      try {
        final synced = await _remote.create(product);

        // Update local with server-confirmed data
        await _local.update(synced);
        await _local.markSynced(synced.productUuid);

        return synced;
      } on NetworkException {
        // Offline, already saved locally
        print('[ProductRepository] Offline - product queued for sync');
      } catch (e) {
        // Server error but already saved locally
        print('[ProductRepository] Sync failed but saved locally: $e');
      }
    }

    // Product saved locally, will sync in background
    return product;
  }

  @override
  Future<Product> update(Product product) async {
    // 1. Update locally FIRST
    await _local.update(product);

    // 2. Try to sync to server
    if (await _isOnline) {
      try {
        final synced = await _remote.update(product);

        // Update local with server-confirmed data
        await _local.update(synced);
        await _local.markSynced(synced.productUuid);

        return synced;
      } on NetworkException {
        print('[ProductRepository] Offline - update queued for sync');
      } catch (e) {
        print('[ProductRepository] Sync failed but saved locally: $e');
      }
    }

    return product;
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
        print('[ProductRepository] Offline - deletion queued for sync');
      } catch (e) {
        print('[ProductRepository] Sync failed but deleted locally: $e');
      }
    }
  }

  @override
  Future<List<Product>> search(String query) async {
    // Search local cache first (instant)
    final localResults = await _local.search(query);

    // If online, also search remote and merge
    if (await _isOnline) {
      try {
        final remoteResults = await _remote.search(query);

        // Cache remote results
        for (final product in remoteResults) {
          await _local.insert(product);
          await _local.markSynced(product.productUuid);
        }

        return remoteResults;
      } catch (e) {
        print('[ProductRepository] Remote search failed: $e');
      }
    }

    return localResults;
  }

  @override
  Future<List<Product>> refresh() async {
    // Force fetch from server
    if (!await _isOnline) {
      throw NetworkException('Cannot refresh while offline');
    }

    final products = await _remote.getAll();

    // Update local cache SAFELY
    final dirtyUuids = await _local.getDirtyUuids();
    final safeProducts =
        products.where((p) => !dirtyUuids.contains(p.productUuid)).toList();

    await _local.insertBatch(safeProducts);

    return await _local.getAll();
  }

  @override
  Future<int> syncPendingChanges() async {
    if (!await _isOnline) {
      return 0;
    }

    final unsynced = await _local.getUnsynced();
    int synced = 0;

    for (final product in unsynced) {
      try {
        if (product.deletedAt != null) {
          // Sync deletion
          await _remote.delete(product.productUuid);
          await _local.hardDelete(product.productUuid);
        } else {
          // Try to update (might be new or modified)
          try {
            await _remote.update(product);
          } catch (e) {
            // If update fails, try create
            await _remote.create(product);
          }
          await _local.markSynced(product.productUuid);
        }
        synced++;
      } catch (e) {
        print('[ProductRepository] Failed to sync ${product.productUuid}: $e');
      }
    }

    return synced;
  }

  @override
  Future<void> clearCache() async {
    await _local.clearAll();
  }

  // === FilterableRepository Methods ===

  @override
  Future<List<Product>> getByCategory(String category) async {
    // Check local first
    final localResults = await _local.getByCategory(category);

    if (await _isOnline) {
      try {
        final remoteResults = await _remote.getByCategory(category);
        await _local.insertBatch(remoteResults);
        return remoteResults;
      } catch (e) {
        print('[ProductRepository] Remote getByCategory failed: $e');
      }
    }

    return localResults;
  }

  @override
  Future<List<Product>> getWhere(Map<String, dynamic> criteria) async {
    // For now, delegate to local DB
    // Can be enhanced with remote filtering
    return await _local.getAll();
  }

  // === SyncableRepository Methods ===

  @override
  Future<List<Product>> getUnsynced() async {
    return await _local.getUnsynced();
  }

  @override
  Future<List<Product>> getFailedSync() async {
    // TODO: Implement failed sync tracking
    return [];
  }

  @override
  Future<void> retrySync(String id) async {
    final product = await _local.getById(id);
    if (product == null) return;

    try {
      await _remote.update(product);
      await _local.markSynced(id);
    } catch (e) {
      print('[ProductRepository] Retry sync failed for $id: $e');
      rethrow;
    }
  }

  // === Additional Helper Methods ===

  /// Get product by barcode
  Future<Product?> getByBarcode(String barcode) async {
    if (await _isOnline) {
      try {
        final product = await _remote.getByBarcode(barcode);
        if (product != null) {
          await _local.insert(product);
          await _local.markSynced(product.productUuid);
        }
        return product;
      } catch (e) {
        print('[ProductRepository] Barcode lookup failed: $e');
      }
    }

    // Fallback to local search by barcode
    // (Requires barcode field in local schema)
    return null;
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
